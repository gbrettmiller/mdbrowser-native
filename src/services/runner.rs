//! Executes the session's effects. Each becomes an iced task whose result,
//! if any, is fed back to the reducer as an event.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::Task;

use super::store::{Store, Ticket};
use super::{background, files, images, opener, web};
use crate::session::error::ErrorCode;
use crate::session::location::Location;
use crate::session::web::Origin;
use crate::session::{Effect, Event, Loaded};

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

/// The built-in page, bundled in the binary.
const WELCOME_PAGE: &str = include_str!("../../assets/welcome.md");

/// Reads a file or fetches a page. Matched exhaustively, so a web location can never reach `files`.
pub fn load(location: Location, limits: &web::Limits, origin: Origin) -> Result<Loaded, ErrorCode> {
    match location {
        Location::File(path) => files::read_markdown(&path).map(|source| Loaded {
            source,
            location: Location::File(path),
            fallback_from: None,
        }),
        Location::Welcome => {
            Ok(Loaded { source: WELCOME_PAGE.to_string(), location: Location::Welcome, fallback_from: None })
        }
        Location::Web(url) => web::fetch(&url, limits, origin).map(|fetched| Loaded {
            source: fetched.source,
            location: Location::web(fetched.url),
            fallback_from: fetched.fallback_from.map(|url| Location::web(url).address()),
        }),
    }
}

pub fn run(effect: Effect, store: Option<Arc<Store>>) -> Task<Option<Event>> {
    match effect {
        Effect::Load { request_id, location, origin } => Task::perform(
            background::run_blocking(move || load(location, &web::Limits::default(), origin)),
            move |result| Some(Event::Loaded { request_id, result, now: now() }),
        ),
        Effect::LoadImage { location, origin } => {
            let requested = location.clone();
            Task::perform(background::run_blocking(move || images::load(location, origin)), move |result| {
                Some(Event::ImageLoaded { location: requested, result })
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
