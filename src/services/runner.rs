//! Executes the session's effects. Each becomes an iced task whose result,
//! if any, is fed back to the reducer as an event.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::Task;

use super::store::Store;
use super::{files, opener};
use crate::session::{Effect, Event};

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

fn in_background(
    store: Option<Arc<Store>>,
    save: impl FnOnce(&Store) -> std::io::Result<()> + Send + 'static,
) -> Task<Option<Event>> {
    match store {
        Some(store) => Task::perform(async move { save(&store) }, |result| {
            if let Err(error) = result {
                eprintln!("mdbrowser: couldn't save: {error}");
            }
            None
        }),
        None => Task::none(),
    }
}

pub fn run(effect: Effect, store: Option<Arc<Store>>) -> Task<Option<Event>> {
    match effect {
        Effect::Load { request_id, location } => {
            Task::perform(async move { files::read_markdown(location.path()) }, move |result| {
                Some(Event::Loaded { request_id, result, now: now() })
            })
        }
        Effect::OpenExternal(url) => Task::perform(async move { opener::open(&url) }, |result| {
            result.err().map(|error| {
                eprintln!("mdbrowser: {error}");
                Event::OpenerFailed
            })
        }),
        Effect::PersistVisits(visits) => in_background(store, move |store| store.save_visits(&visits)),
        Effect::PersistSettings(settings) => in_background(store, move |store| store.save_settings(settings)),
    }
}
