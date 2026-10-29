//! The acceptance world: a temporary home, the real reducer, real file reads,
//! and fake opener/store so effects can be observed.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use mdbrowser::services::files;
use mdbrowser::services::store::Store;
use mdbrowser::session::visits::Visit;
use mdbrowser::session::{Effect, Event, State, step};
use tempfile::TempDir;

const INDEX: &str = "---
title: Home
---
# Welcome

## Intro

[A](notes/a.md) [intro](#intro) [gone](gone.md) [data](data.txt)
[ex](https://example.com) [mail](mailto:a@b.co) [ftp](ftp://x.org/f)
";

const NOTE_A: &str = "# Note A

[home](../index.md#intro)
";

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
pub struct Browser {
    pub home: TempDir,
    state: Option<State>,
    /// Every path a load effect actually read.
    pub reads: Vec<PathBuf>,
    pub opened: Vec<String>,
    pub opener_fails: bool,
    /// Loads for these paths wait until released.
    pub held: HashSet<PathBuf>,
    parked: Vec<(u64, PathBuf)>,
    /// The real store, kept in the temporary home.
    pub store: Store,
    clock: u64,
}

impl Browser {
    fn new() -> Self {
        let home = tempfile::tempdir().expect("temp home");
        fs::create_dir_all(home.path().join("notes")).unwrap();
        fs::write(home.path().join("index.md"), INDEX).unwrap();
        fs::write(home.path().join("notes/a.md"), NOTE_A).unwrap();
        fs::write(home.path().join("data.txt"), "not markdown").unwrap();
        let state = State::new(home.path());
        let store = Store::at(home.path().join(".mdbrowser"));
        Browser {
            home,
            state: Some(state),
            reads: Vec::new(),
            opened: Vec::new(),
            opener_fails: false,
            held: HashSet::new(),
            parked: Vec::new(),
            store,
            clock: 0,
        }
    }

    pub fn state(&self) -> &State {
        self.state.as_ref().expect("state")
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.home.path().join(name)
    }

    pub fn absolute(&self, name: &str) -> String {
        self.path(name).display().to_string()
    }

    pub fn send(&mut self, event: Event) {
        let (state, effects) = step(self.state.take().expect("state"), event);
        self.state = Some(state);
        effects.into_iter().for_each(|effect| self.run(effect));
    }

    fn run(&mut self, effect: Effect) {
        match effect {
            Effect::Load { request_id, location } => {
                let path = location.path().to_path_buf();
                if self.held.contains(&path) {
                    self.parked.push((request_id, path));
                } else {
                    self.finish_load(request_id, path);
                }
            }
            Effect::OpenExternal(url) => {
                if self.opener_fails {
                    self.send(Event::OpenerFailed);
                } else {
                    self.opened.push(url);
                }
            }
            Effect::PersistVisits(visits) => {
                self.store.save_visits(self.store.ticket(), &visits).expect("save history")
            }
            Effect::PersistSettings(settings) => {
                self.store.save_settings(self.store.ticket(), settings).expect("save settings")
            }
        }
    }

    fn finish_load(&mut self, request_id: u64, path: PathBuf) {
        self.reads.push(path.clone());
        self.clock += 1;
        let result = files::read_markdown(&path);
        self.send(Event::Loaded { request_id, result, now: self.clock });
    }

    /// Lets a held load complete, with the request id it was issued under.
    pub fn release(&mut self, name: &str) {
        let path = self.path(name);
        self.held.remove(&path);
        let (ready, still): (Vec<_>, Vec<_>) = self.parked.drain(..).partition(|(_, p)| *p == path);
        self.parked = still;
        ready.into_iter().for_each(|(id, p)| self.finish_load(id, p));
    }

    pub fn restart(&mut self) {
        self.state = Some(State::new(self.home.path()));
        let visits = self.store.load_visits();
        let settings = self.store.load_settings();
        self.send(Event::Restore { visits, settings });
    }

    pub fn seed_history(&mut self, visits: Vec<Visit>) {
        self.store.save_visits(self.store.ticket(), &visits).expect("seed history");
        self.restart();
    }

    pub fn history_titles(&self) -> Vec<String> {
        self.state().visits.iter().map(|v| v.title.clone()).collect()
    }
}
