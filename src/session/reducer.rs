//! `step`: every way the session can change, as one pure function.

use std::sync::Arc;

use super::address::parse_address;
use super::error::ErrorCode;
use super::images::Images;
use super::links::{self, Link};
use super::location::Location;
use super::navigation::{Entry, Navigation};
use super::visits;
use super::web::Origin;
use super::{Effect, Event, Loaded, Notice, NoticeKind, Settings, Shown, State, Status};
use crate::model;

type Step = (State, Vec<Effect>);

fn none(state: State) -> Step {
    (state, Vec::new())
}

pub fn step(state: State, event: Event) -> Step {
    match event {
        Event::Enter(input) | Event::OpenHistory(input) => {
            let entry = entry_for(&input, &state);
            navigate_to(state, entry)
        }
        Event::Follow(href) => follow(state, &href),
        Event::Back => move_by(state, Navigation::back),
        Event::Forward => move_by(state, Navigation::forward),
        Event::Reload => match state.nav.current().cloned() {
            Some(entry) => enter(state, entry),
            None => none(state),
        },
        Event::Loaded { request_id, result, now } => loaded(state, request_id, result, now),
        Event::OpenerFailed => notice(state, NoticeKind::OpenFailed),
        Event::OpenInBrowser => match state.browser_offer() {
            Some(url) => (state, vec![Effect::OpenExternal(url)]),
            None => none(state),
        },
        Event::ClearHistory => persist_visits(State { visits: Vec::new(), ..state }),
        Event::ChooseFont(font) => persist_settings(State { settings: Settings { font, ..state.settings }, ..state }),
        Event::ChooseImageLoading(mode) => {
            let (state, mut effects) = with_images(state, |images| images.mode_changed(mode));
            let (state, persist) =
                persist_settings(State { settings: Settings { images: mode, ..state.settings }, ..state });
            effects.extend(persist);
            (state, effects)
        }
        Event::ImageLoaded { location, result } => with_images(state, |images| images.loaded(location, result)),
        Event::LoadImage(src) => match state.images.location(&src).cloned() {
            Some(location) => with_images(state, |images| images.click(&location)),
            None => none(state),
        },
        Event::LoadPageImages => with_images(state, Images::load_page),
        Event::DismissNotice(id) => {
            let notices = state.notices.iter().copied().filter(|n| n.id != id).collect();
            none(State { notices, ..state })
        }
        Event::FragmentHandled => none(State { fragment: None, ..state }),
        Event::Restore { visits, settings } => none(State { visits, settings, ..state }),
    }
}

/// What gets pushed for typed input: the canonical address if it parses,
/// otherwise the raw input, so the error can be shown and revisited.
fn entry_for(input: &str, state: &State) -> Entry {
    match parse_address(input, &state.home) {
        Ok(target) => Entry { input: target.location.address(), target: Some(target), origin: Origin::Trusted },
        Err(_) => Entry { input: input.to_string(), target: None, origin: Origin::Trusted },
    }
}

fn navigate_to(state: State, entry: Entry) -> Step {
    let nav = state.nav.clone().push(entry.clone());
    enter(State { nav, ..state }, entry)
}

fn move_by(state: State, mover: fn(Navigation) -> Navigation) -> Step {
    let nav = mover(state.nav.clone());
    match nav.current().cloned() {
        Some(entry) if nav != state.nav => enter(State { nav, ..state }, entry),
        _ => none(state),
    }
}

/// Shows an entry. Every call bumps the request id, so any load in flight goes stale,
/// and leaves the page's images: their queued loads are dropped.
fn enter(state: State, entry: Entry) -> Step {
    let request_id = state.request_id + 1;
    let state = State { images: state.images.leave_page(), ..state };
    match entry.target {
        Some(target) => {
            let state = State { request_id, pending: Some(request_id), status: Status::Loading, ..state };
            (state, vec![Effect::Load { request_id, location: target.location, origin: entry.origin }])
        }
        None => {
            let code = parse_address(&entry.input, &state.home).err().unwrap_or(ErrorCode::NotAbsolute);
            none(failed(State { request_id, ..state }, code))
        }
    }
}

fn failed(state: State, code: ErrorCode) -> State {
    State { pending: None, status: Status::Failed(code), fragment: None, ..state }
}

/// A finished load for the current entry. A redirect settles the entry on the
/// final location, so the address bar, History and relative links all use it.
fn loaded(state: State, request_id: u64, result: Result<Loaded, ErrorCode>, now: u64) -> Step {
    let target = state.nav.current().and_then(|entry| entry.target.clone());
    let (Some(target), true) = (target, state.pending == Some(request_id)) else { return none(state) };
    match result {
        Ok(Loaded { source, location }) => {
            let document = model::parse(&source);
            let title = document.title(&location.label());
            let visits = visits::record(&state.visits, &location.address(), &title, now);
            let shown = Arc::new(Shown { document: Arc::new(document), title });
            let sources = shown.document.image_sources();
            let (images, loads) = state.images.open_page(&sources, &location, &state.home, state.settings.images);
            let origin = Origin::of(&location);
            let nav = if location == target.location { state.nav } else { state.nav.settle(location) };
            let status = Status::Ready(shown);
            let state = State { nav, visits, images, pending: None, status, fragment: target.fragment, ..state };
            let (state, mut effects) = persist_visits(state);
            effects.extend(loads.into_iter().map(|location| Effect::LoadImage { location, origin }));
            (state, effects)
        }
        Err(code) => none(failed(state, code)),
    }
}

fn follow(state: State, href: &str) -> Step {
    let Some(location) = state.current_location().filter(|_| matches!(state.status, Status::Ready(_))) else {
        return none(state);
    };
    match links::classify(href, location) {
        Link::Navigate(target) => {
            let origin = Origin::of(location);
            navigate_to(state, Entry { input: target.location.address(), target: Some(target), origin })
        }
        Link::Scroll(fragment) => none(State { fragment: Some(fragment), ..state }),
        Link::External(url) => (state, vec![Effect::OpenExternal(url)]),
        Link::LocalFromWeb => notice(state, NoticeKind::LocalFromWeb),
        Link::LocalNetworkFromWeb => notice(state, NoticeKind::LocalNetworkFromWeb),
        Link::Unsupported => notice(state, NoticeKind::UnsupportedLink),
    }
}

fn notice(state: State, kind: NoticeKind) -> Step {
    let id = state.notice_seq + 1;
    let notices = state.notices.iter().copied().chain([Notice { id, kind }]).collect();
    none(State { notice_seq: id, notices, ..state })
}

fn persist_visits(state: State) -> Step {
    let effects = vec![Effect::PersistVisits(state.visits.clone())];
    (state, effects)
}

/// Applies a change to the images and turns the loads it starts into effects,
/// fetched under the current page's origin (ADR-004).
fn with_images(state: State, change: impl FnOnce(Images) -> (Images, Vec<Location>)) -> Step {
    let origin = state.current_location().map_or(Origin::Trusted, Origin::of);
    let (images, loads) = change(state.images.clone());
    let effects = loads.into_iter().map(|location| Effect::LoadImage { location, origin }).collect();
    (State { images, ..state }, effects)
}

fn persist_settings(state: State) -> Step {
    let effects = vec![Effect::PersistSettings(state.settings)];
    (state, effects)
}
