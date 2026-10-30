//! The acceptance world: a temporary home, a local web server, the real reducer,
//! real loads, and a fake opener so effects can be observed.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mdbrowser::services::store::Store;
use mdbrowser::services::web::Limits;
use mdbrowser::services::{images, runner};
use mdbrowser::session::location::Location;
use mdbrowser::session::visits::Visit;
use mdbrowser::session::web::Origin;
use mdbrowser::session::{Effect, Event, State, step};
use tempfile::TempDir;
use url::Url;

use crate::fixtures;
use crate::http::{Reply, Server};

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

/// Small enough that `/huge.md` can exceed it cheaply.
const SIZE_LIMIT: u64 = 4096;

const REMOTE_INDEX: &str = "# Remote Home

## Intro

[A](notes/a.md) [B](/notes/b.md) [intro](#intro) [gone](/missing.md)
[about](https://example.com/about) [mail](mailto:a@b.co)
";

/// The routes described at the top of features/remote.feature.
fn routes() -> HashMap<String, Reply> {
    let typed = |content_type: &str, body: &[u8]| Reply::new(200).header("Content-Type", content_type).body(body);
    [
        ("/index.md", Reply::markdown(REMOTE_INDEX)),
        ("/notes/a.md", Reply::markdown("# Remote Note\n")),
        ("/notes/b.md", Reply::markdown("# Note B\n")),
        ("/post", Reply::markdown("# Post Without Extension\n")),
        ("/plain.md", typed("text/plain", b"# Plain Served\n")),
        ("/plain.txt", typed("text/plain", b"# Not this\n")),
        ("/untyped.md", Reply::new(200).body("# Untyped\n")),
        ("/page.html", typed("text/html", b"<h1>HTML</h1>")),
        ("/fake.md", typed("text/html", b"<h1>Not Markdown</h1>")),
        ("/moved.md", Reply::redirect(301, "/index.md")),
        ("/loop.md", Reply::redirect(302, "/loop.md")),
        ("/to-file.md", Reply::redirect(302, "file:///etc/hosts")),
        ("/no-location.md", Reply::new(302)),
        ("/gone.md", Reply::new(410)),
        ("/broken.md", Reply::new(500)),
        ("/latin1.md", typed("text/markdown; charset=iso-8859-1", b"# Caf\xe9\n")),
        ("/bad-utf8.md", typed("text/markdown", b"# \xff\xfe\n")),
        ("/huge.md", Reply::markdown(&"x".repeat(SIZE_LIMIT as usize + 1))),
        ("/slow.md", Reply::markdown("# Slow\n")),
        ("/article", typed("text/html", b"<h1>Article</h1>")),
        ("/article.md", Reply::markdown("# Article In Markdown\n")),
        (
            "/linked",
            typed("text/html", b"<h1>Linked</h1>")
                .header("Link", "</alt/linked.md>; rel=\"alternate\"; type=\"text/markdown\""),
        ),
        ("/alt/linked.md", Reply::markdown("# Linked Alternate\n")),
        ("/linked.md", Reply::markdown("# Sibling Loses\n")),
        ("/bare", typed("text/html", b"<h1>Bare</h1>")),
    ]
    .into_iter()
    .chain(fixtures::image_routes())
    .map(|(path, reply)| (path.to_string(), reply))
    .collect()
}

/// Whether a web load stays on the scenario's own server: nothing reaches the internet.
pub fn on_test_server(url: &Url) -> bool {
    url.host_str() == Some("127.0.0.1")
}

/// Fails the first time it is asked for, then serves an image.
const FLAKY: &str = "/img/flaky.png";

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
pub struct Browser {
    pub home: TempDir,
    state: Option<State>,
    /// Every path a load effect actually read.
    pub reads: Vec<PathBuf>,
    pub opened: Vec<String>,
    /// What the last "select everything and copy" copied.
    pub copied: Option<String>,
    pub opener_fails: bool,
    /// Loads of these locations wait until released.
    held: HashSet<Location>,
    parked: Vec<(u64, Location, Origin)>,
    /// Web image loads under these paths wait; started ones wait here.
    slow_images: Vec<String>,
    parked_images: Vec<(Location, Origin)>,
    pub server: Server,
    /// What the server answers, by path; anything else is a 404.
    pub routes: Arc<Mutex<HashMap<String, Reply>>>,
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
        let routes = Arc::new(Mutex::new(routes()));
        let served = Arc::clone(&routes);
        let flaky_seen = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server = Server::start(move |request| {
            if request.path == FLAKY {
                let seen = flaky_seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return if seen == 0 {
                    Reply::new(500)
                } else {
                    Reply::new(200).header("Content-Type", "image/png").body(fixtures::png(40, 20))
                };
            }
            let routes = served.lock().expect("routes");
            routes.get(&request.path).cloned().unwrap_or_else(|| Reply::new(404))
        });
        Browser {
            home,
            state: Some(state),
            reads: Vec::new(),
            opened: Vec::new(),
            copied: None,
            opener_fails: false,
            held: HashSet::new(),
            parked: Vec::new(),
            slow_images: Vec::new(),
            parked_images: Vec::new(),
            server,
            routes,
            store,
            clock: 0,
        }
    }

    pub fn state(&self) -> &State {
        self.state.as_ref().expect("state")
    }

    /// Feature files name documents with `/`; build the path with the platform's separator.
    pub fn path(&self, name: &str) -> PathBuf {
        name.split('/').fold(self.home.path().to_path_buf(), |path, part| path.join(part))
    }

    /// The document's `file://` URL: `file:///C:/…` on Windows, `file:///home/…` elsewhere.
    pub fn file_url(&self, name: &str) -> String {
        let path = self.absolute(name).replace('\\', "/");
        let slash = if path.starts_with('/') { "" } else { "/" };
        format!("file://{slash}{path}")
    }

    /// The test server's URL for `path`.
    pub fn url(&self, path: &str) -> String {
        self.server.url(path)
    }

    pub fn web(&self, path: &str) -> Location {
        Location::web(Url::parse(&self.url(path)).expect("a test URL"))
    }

    /// A step's name for something to load: a server path (`/x.md`) or a file in the home.
    fn location_named(&self, name: &str) -> Location {
        if name.starts_with('/') { self.web(name) } else { Location::file(self.path(name)) }
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
            Effect::Load { request_id, location, origin } => {
                if self.held.contains(&location) {
                    self.parked.push((request_id, location, origin));
                } else {
                    self.finish_load(request_id, location, origin);
                }
            }
            Effect::LoadImage { location, origin } => self.finish_image(location, origin),
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

    /// Loads through the app's own `runner::load`, with a lowered size limit.
    fn finish_load(&mut self, request_id: u64, location: Location, origin: Origin) {
        match &location {
            Location::File(path) => self.reads.push(path.clone()),
            Location::Web(url) => assert!(on_test_server(url), "scenarios never leave the test server: {url}"),
            Location::Welcome => {}
        }
        self.clock += 1;
        let result = runner::load(location, &Limits { max_bytes: SIZE_LIMIT, ..Limits::default() }, origin);
        self.send(Event::Loaded { request_id, result, now: self.clock });
    }

    /// Loads an image through the app's own `services::images::load`.
    fn finish_image(&mut self, location: Location, origin: Origin) {
        let held = match &location {
            Location::Web(url) => self.slow_images.iter().any(|prefix| url.path().starts_with(prefix.as_str())),
            Location::File(_) | Location::Welcome => false,
        };
        if held {
            self.parked_images.push((location, origin));
        } else {
            self.fetch_image(location, origin);
        }
    }

    fn fetch_image(&mut self, location: Location, origin: Origin) {
        match &location {
            Location::File(path) => self.reads.push(path.clone()),
            Location::Web(url) => assert!(on_test_server(url), "scenarios never leave the test server: {url}"),
            Location::Welcome => {}
        }
        let result = images::load(location.clone(), origin);
        self.send(Event::ImageLoaded { location, result });
    }

    /// Web image loads whose path starts with `prefix` wait until released.
    pub fn hold_images_under(&mut self, prefix: &str) {
        self.slow_images.push(prefix.to_string());
    }

    /// Image loads started and not yet finished.
    pub fn waiting_images(&self) -> usize {
        self.parked_images.len()
    }

    /// Lets every waiting image load finish, in the order they started.
    pub fn release_images(&mut self) {
        self.slow_images.clear();
        let waiting = std::mem::take(&mut self.parked_images);
        waiting.into_iter().for_each(|(location, origin)| self.fetch_image(location, origin));
    }

    /// Lets the waiting load of one image finish; returns false if none was waiting.
    fn release_image(&mut self, path: &str) -> bool {
        let location = self.web(path);
        let Some(index) = self.parked_images.iter().position(|(held, _)| *held == location) else { return false };
        let (location, origin) = self.parked_images.remove(index);
        self.fetch_image(location, origin); // loads it starts are still held
        true
    }

    /// The session's image cache, with room for `bytes` of pixels.
    pub fn set_image_budget(&mut self, bytes: u64) {
        let state = self.state.take().expect("state");
        self.state = Some(State { images: mdbrowser::session::images::Images::with_budget(bytes), ..state });
    }

    pub fn hold(&mut self, name: &str) {
        let location = self.location_named(name);
        self.held.insert(location);
    }

    /// Lets a held load complete, with the request id it was issued under.
    pub fn release(&mut self, name: &str) {
        if name.starts_with('/') && self.release_image(name) {
            return;
        }
        let location = self.location_named(name);
        self.held.remove(&location);
        let (ready, still): (Vec<_>, Vec<_>) = self.parked.drain(..).partition(|(_, held, _)| *held == location);
        assert_eq!(ready.len(), 1, "exactly one load of {name} was waiting");
        self.parked = still;
        ready.into_iter().for_each(|(id, held, origin)| self.finish_load(id, held, origin));
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
