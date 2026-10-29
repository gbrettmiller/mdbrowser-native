//! Executes the session's effects. Each becomes an iced task whose result,
//! if any, is fed back to the reducer as an event.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::Task;

use super::store::{Store, Ticket};
use super::{background, files, opener};
use crate::session::error::ErrorCode;
use crate::session::location::Location;
use crate::session::{Effect, Event};

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

/// Takes the store's ticket now, while effects are still in dispatch order,
/// then saves in the background.
fn in_background(
    store: Option<Arc<Store>>,
    save: impl FnOnce(&Store, Ticket) -> std::io::Result<()> + Send + 'static,
) -> Task<Option<Event>> {
    match store {
        Some(store) => {
            let ticket = store.ticket();
            Task::perform(background::run_blocking(move || save(&store, ticket)), |result| {
                if let Err(error) = result {
                    eprintln!("mdbrowser: couldn't save: {error}");
                }
                None
            })
        }
        None => Task::none(),
    }
}

pub fn run(effect: Effect, store: Option<Arc<Store>>) -> Task<Option<Event>> {
    match effect {
        Effect::Load { request_id, location } => {
            let load = move || match location {
                Location::File(path) => files::read_markdown(&path),
                Location::Web(_) => Err(ErrorCode::Unreachable), // fetching arrives with services::web
            };
            Task::perform(background::run_blocking(load), move |result| {
                Some(Event::Loaded { request_id, result, now: now() })
            })
        }
        Effect::OpenExternal(url) => Task::perform(background::run_blocking(move || opener::open(&url)), |result| {
            result.err().map(|error| {
                eprintln!("mdbrowser: {error}");
                Event::OpenerFailed
            })
        }),
        Effect::PersistVisits(visits) => in_background(store, move |store, ticket| store.save_visits(ticket, &visits)),
        Effect::PersistSettings(settings) => {
            in_background(store, move |store, ticket| store.save_settings(ticket, settings))
        }
    }
}
