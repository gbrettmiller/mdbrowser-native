//! The reducer, driven through `step` with every kind of event.

use super::*;
use crate::session::address::native;
use crate::session::images::ImageLoading;

fn run(events: impl IntoIterator<Item = Event>) -> (State, Vec<Effect>) {
    events.into_iter().fold((State::new(native("/home/r")), Vec::new()), |(state, _), event| step(state, event))
}

fn loaded_at(request_id: u64, location: Location, source: &str) -> Event {
    Event::Loaded { request_id, result: Ok(Loaded { source: source.into(), location }), now: 1 }
}

fn loaded(request_id: u64, path: &str, source: &str) -> Event {
    loaded_at(request_id, Location::file(native(path)), source)
}

fn web(url: &str) -> Location {
    Location::web(url::Url::parse(url).unwrap())
}

fn fetched(request_id: u64, url: &str, source: &str) -> Event {
    loaded_at(request_id, web(url), source)
}

fn history(effects: &[Effect]) -> Vec<String> {
    match effects {
        [Effect::PersistVisits(visits)] => visits.iter().map(|visit| visit.address.clone()).collect(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn entering_an_address_requests_a_load_and_waits() {
    let (state, effects) = run([Event::Enter(native("/d/a.md"))]);
    assert_eq!(state.status, Status::Loading);
    assert_eq!(effects, [Effect::Load { request_id: 1, location: Location::file(native("/d/a.md")) }]);
}

#[test]
fn a_load_shows_the_document_and_records_history() {
    let (state, effects) = run([Event::Enter(format!("{}#intro", native("/d/a.md"))), loaded(1, "/d/a.md", "# A")]);
    let Status::Ready(shown) = &state.status else { panic!("{:?}", state.status) };
    assert_eq!(shown.title, "A");
    assert_eq!(state.fragment.as_deref(), Some("intro"));
    assert!(matches!(&effects[..], [Effect::PersistVisits(visits)] if visits[0].address == native("/d/a.md")));
}

#[test]
fn stale_loads_are_ignored() {
    let (state, effects) =
        run([Event::Enter(native("/d/a.md")), Event::Enter(native("/d/b.md")), loaded(1, "/d/a.md", "# A")]);
    assert_eq!(state.status, Status::Loading);
    assert!(effects.is_empty());
}

#[test]
fn entering_a_web_address_requests_a_load_and_waits() {
    let (state, effects) = run([Event::Enter("https://x.org/a.md".into())]);
    assert_eq!(state.status, Status::Loading);
    assert_eq!(effects, [Effect::Load { request_id: 1, location: web("https://x.org/a.md") }]);
}

#[test]
fn a_redirected_load_settles_on_its_final_address() {
    let (state, effects) =
        run([Event::Enter("https://x.org/old.md#intro".into()), fetched(1, "https://x.org/new.md", "# New")]);
    assert_eq!(state.address(), "https://x.org/new.md");
    assert_eq!(state.current_location(), Some(&web("https://x.org/new.md")));
    assert_eq!(state.fragment.as_deref(), Some("intro"));
    assert_eq!(history(&effects), ["https://x.org/new.md"]);
}

#[test]
fn a_redirect_to_a_visited_address_moves_that_visit_to_the_top() {
    let (_, effects) = run([
        Event::Enter("https://x.org/b.md".into()),
        fetched(1, "https://x.org/b.md", "# B"),
        Event::Enter("https://x.org/c.md".into()),
        fetched(2, "https://x.org/c.md", "# C"),
        Event::Enter("https://x.org/old.md".into()),
        fetched(3, "https://x.org/b.md", "# B"),
    ]);
    assert_eq!(history(&effects), ["https://x.org/b.md", "https://x.org/c.md"]);
}

#[test]
fn a_stale_redirected_load_settles_nothing() {
    let (state, effects) = run([
        Event::Enter("https://x.org/old.md".into()),
        Event::Enter("https://x.org/b.md".into()),
        fetched(1, "https://x.org/new.md", "# New"),
    ]);
    assert_eq!((state.status.clone(), state.address()), (Status::Loading, "https://x.org/b.md".into()));
    assert!(effects.is_empty());
}

#[test]
fn a_failed_web_load_keeps_the_typed_address_and_records_nothing() {
    let failure = Event::Loaded { request_id: 1, result: Err(ErrorCode::NotFound), now: 1 };
    let (state, effects) = run([Event::Enter("https://x.org/a.md".into()), failure]);
    assert_eq!(state.status, Status::Failed(ErrorCode::NotFound));
    assert_eq!(state.address(), "https://x.org/a.md");
    assert!(effects.is_empty() && state.visits.is_empty());
}

#[test]
fn going_back_to_or_reloading_a_settled_entry_loads_its_final_address() {
    let (_, effects) =
        run([Event::Enter("https://x.org/old.md".into()), fetched(1, "https://x.org/new.md", "# New"), Event::Reload]);
    assert_eq!(effects, [Effect::Load { request_id: 2, location: web("https://x.org/new.md") }]);
    let (_, effects) = run([
        Event::Enter("https://x.org/old.md".into()),
        fetched(1, "https://x.org/new.md", "# New"),
        Event::Enter("https://x.org/c.md".into()),
        Event::Back,
    ]);
    assert_eq!(effects, [Effect::Load { request_id: 3, location: web("https://x.org/new.md") }]);
}

#[test]
fn links_do_nothing_until_a_document_is_showing() {
    let (state, effects) = run([Event::Enter(native("/d/a.md")), Event::Follow("https://x.org".into())]);
    assert!(effects.is_empty() && state.notices.is_empty());
}

#[test]
fn links_a_page_may_not_open_become_notices() {
    let opened = [Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "# A")];
    let (state, _) = run(opened.clone().into_iter().chain([Event::Follow("file:///etc/x.md".into())]));
    assert_eq!(state.notices.last().map(|n| n.kind), Some(NoticeKind::LocalFromWeb));
    let (state, _) = run(opened.into_iter().chain([Event::Follow("http://127.0.0.1/x.md".into())]));
    assert_eq!(state.notices.last().map(|n| n.kind), Some(NoticeKind::LocalNetworkFromWeb));
    assert_eq!(state.address(), "https://x.org/a.md");
}

fn failed_with(code: ErrorCode) -> Event {
    Event::Loaded { request_id: 1, result: Err(code), now: 1 }
}

#[test]
fn a_page_that_is_not_markdown_can_go_to_the_system_browser() {
    let (state, effects) = run([Event::Enter("https://x.org/page.html".into()), failed_with(ErrorCode::NotMarkdown)]);
    assert_eq!(state.browser_offer().as_deref(), Some("https://x.org/page.html"));
    assert!(effects.is_empty());
    let (_, effects) = step(state, Event::OpenInBrowser);
    assert_eq!(effects, [Effect::OpenExternal("https://x.org/page.html".into())]);
}

#[test]
fn the_system_browser_is_offered_for_nothing_else() {
    let cases = [
        vec![Event::Enter("https://x.org/a.md".into()), failed_with(ErrorCode::NotFound)],
        vec![Event::Enter(native("/d/a.md")), failed_with(ErrorCode::NotMarkdown)],
        vec![Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "# A")],
        vec![Event::Enter("https://x.org/a.md".into())],
        vec![Event::Enter("ftp://x.org/a.md".into())],
        vec![],
    ];
    for events in cases {
        let (state, _) = run(events.clone());
        assert_eq!(state.browser_offer(), None, "{events:?}");
        let (_, effects) = step(state, Event::OpenInBrowser);
        assert!(effects.is_empty(), "{events:?}");
    }
}

#[test]
fn notices_accumulate_and_dismiss_by_id() {
    let (state, _) = run([Event::OpenerFailed, Event::OpenerFailed, Event::DismissNotice(1)]);
    assert_eq!(state.notices, [Notice { id: 2, kind: NoticeKind::OpenFailed }]);
}

#[test]
fn choosing_a_font_persists_it_and_restore_brings_it_back() {
    let (state, effects) = run([Event::ChooseFont(ReadingFont::Inter)]);
    let settings = Settings { font: ReadingFont::Inter, ..Settings::default() };
    assert_eq!((state.settings, effects), (settings, vec![Effect::PersistSettings(settings)]));
    let (restored, _) = run([Event::Restore { visits: vec![], settings }]);
    assert_eq!(restored.settings.font, ReadingFont::Inter);
}

#[test]
fn choosing_how_web_images_load_persists_it_and_keeps_the_font() {
    let (state, effects) =
        run([Event::ChooseFont(ReadingFont::Inter), Event::ChooseImageLoading(ImageLoading::OnClick)]);
    let settings = Settings { font: ReadingFont::Inter, images: ImageLoading::OnClick };
    assert_eq!((state.settings, effects), (settings, vec![Effect::PersistSettings(settings)]));
}
