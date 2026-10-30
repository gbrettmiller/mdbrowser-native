//! The reducer, driven through `step` with every kind of event.

use super::*;
use crate::session::address::native;
use crate::session::images::{ImageLoading, Note, Problem, View};
use crate::session::web::Origin;
use crate::theme::Appearance;

fn run(events: impl IntoIterator<Item = Event>) -> (State, Vec<Effect>) {
    events.into_iter().fold((State::new(native("/home/r")), Vec::new()), |(state, _), event| step(state, event))
}

fn loaded_at(request_id: u64, location: Location, source: &str) -> Event {
    Event::Loaded { request_id, result: Ok(Loaded { source: source.into(), location, fallback_from: None }), now: 1 }
}

fn found_instead(request_id: u64, original: &str, url: &str, source: &str) -> Event {
    let loaded = Loaded { source: source.into(), location: web(url), fallback_from: Some(original.into()) };
    Event::Loaded { request_id, result: Ok(loaded), now: 1 }
}

fn shown(state: &State) -> Arc<Shown> {
    match &state.status {
        Status::Ready(shown) => shown.clone(),
        other => panic!("not ready: {other:?}"),
    }
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
    assert_eq!(
        effects,
        [Effect::Load { request_id: 1, location: Location::file(native("/d/a.md")), origin: Origin::Trusted }]
    );
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
    assert_eq!(effects, [Effect::Load { request_id: 1, location: web("https://x.org/a.md"), origin: Origin::Trusted }]);
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
    assert_eq!(
        effects,
        [Effect::Load { request_id: 2, location: web("https://x.org/new.md"), origin: Origin::Trusted }]
    );
    let (_, effects) = run([
        Event::Enter("https://x.org/old.md".into()),
        fetched(1, "https://x.org/new.md", "# New"),
        Event::Enter("https://x.org/c.md".into()),
        Event::Back,
    ]);
    assert_eq!(
        effects,
        [Effect::Load { request_id: 3, location: web("https://x.org/new.md"), origin: Origin::Trusted }]
    );
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
        vec![Event::Enter(native("/d/a.md")), failed_with(ErrorCode::NotMarkdownWithLlms)],
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
    assert_eq!((state.settings, effects), (settings.clone(), vec![Effect::PersistSettings(settings.clone())]));
    let (restored, _) = run([Event::Restore { visits: vec![], settings: settings.clone() }]);
    assert_eq!(restored.settings.font, ReadingFont::Inter);
}

#[test]
fn choosing_how_web_images_load_persists_it_and_keeps_the_font() {
    let (state, effects) =
        run([Event::ChooseFont(ReadingFont::Inter), Event::ChooseImageLoading(ImageLoading::OnClick)]);
    let settings = Settings { font: ReadingFont::Inter, images: ImageLoading::OnClick, ..Settings::default() };
    assert_eq!((state.settings, effects), (settings.clone(), vec![Effect::PersistSettings(settings)]));
}

#[test]
fn choosing_an_appearance_persists_it_and_keeps_the_other_settings() {
    let (state, effects) = run([Event::ChooseFont(ReadingFont::Inter), Event::ChooseAppearance(Appearance::Dark)]);
    let settings = Settings { font: ReadingFont::Inter, appearance: Appearance::Dark, ..Settings::default() };
    assert_eq!((state.settings, effects), (settings.clone(), vec![Effect::PersistSettings(settings)]));
}

#[test]
fn a_home_page_is_stored_as_its_canonical_address() {
    let (state, effects) = run([Event::ChooseHome(" HTTPS://Example.COM/index.md ".into())]);
    assert_eq!(state.settings.home, "https://example.com/index.md");
    assert_eq!(effects, vec![Effect::PersistSettings(state.settings.clone())]);
    let (state, _) = run([Event::ChooseHome(native("/d/start.md"))]);
    assert_eq!(state.settings.home, native("/d/start.md"));
}

#[test]
fn an_empty_home_page_clears_it() {
    let (state, effects) = run([Event::ChooseHome(native("/d/a.md")), Event::ChooseHome("  ".into())]);
    assert_eq!(state.settings.home, "");
    assert_eq!(effects, vec![Effect::PersistSettings(state.settings.clone())]);
}

#[test]
fn an_invalid_home_page_is_refused_and_the_old_one_stays() {
    let (state, effects) = run([Event::ChooseHome(native("/d/a.md")), Event::ChooseHome("notes.txt".into())]);
    assert_eq!(state.settings.home, native("/d/a.md"));
    assert_eq!(state.notices, [Notice { id: 1, kind: NoticeKind::BadHome }]);
    assert_eq!(effects, vec![]);
}

#[test]
fn home_opens_the_home_page_like_any_navigation() {
    let (state, effects) = run([Event::ChooseHome(native("/d/start.md"))]);
    assert_eq!(state.settings.home, native("/d/start.md"));
    assert_eq!(effects, vec![Effect::PersistSettings(state.settings.clone())]);
    let (state, effects) = run([Event::ChooseHome(native("/d/start.md")), Event::Home]);
    assert_eq!(state.address(), native("/d/start.md"));
    assert_eq!(
        effects,
        vec![Effect::Load { request_id: 1, location: Location::file(native("/d/start.md")), origin: Origin::Trusted }]
    );
}

#[test]
fn without_a_home_page_home_is_the_welcome_page_with_a_blank_address() {
    let (state, effects) = run([Event::Home]);
    assert_eq!(state.address(), "");
    assert_eq!(effects, vec![Effect::Load { request_id: 1, location: Location::Welcome, origin: Origin::Trusted }]);
    let (state, effects) = run([Event::Home, loaded_at(1, Location::Welcome, "# Welcome\n\nHello")]);
    let Status::Ready(shown) = &state.status else { panic!("{:?}", state.status) };
    assert_eq!(shown.title, "Welcome");
    assert_eq!(state.address(), "");
    assert!(state.visits.is_empty(), "the welcome page has no address to revisit");
    assert!(history(&effects).is_empty());
}

#[test]
fn launching_without_an_address_opens_the_home_page() {
    let (state, effects) = run([Event::ChooseHome(native("/d/start.md")), Event::Launch(None)]);
    assert_eq!(state.address(), native("/d/start.md"));
    assert_eq!(
        effects,
        vec![Effect::Load { request_id: 1, location: Location::file(native("/d/start.md")), origin: Origin::Trusted }]
    );
    let (state, effects) = run([Event::Launch(None)]);
    assert_eq!(state.address(), "");
    assert_eq!(effects, vec![Effect::Load { request_id: 1, location: Location::Welcome, origin: Origin::Trusted }]);
}

#[test]
fn a_launch_address_wins_over_the_home_page() {
    let (state, effects) = run([Event::ChooseHome(native("/d/start.md")), Event::Launch(Some(native("/d/a.md")))]);
    assert_eq!(state.address(), native("/d/a.md"));
    assert_eq!(
        effects,
        vec![Effect::Load { request_id: 1, location: Location::file(native("/d/a.md")), origin: Origin::Trusted }]
    );
}

#[test]
fn a_home_page_that_fails_shows_the_error_and_the_reader_can_move_on() {
    let missing = Event::Loaded { request_id: 1, result: Err(ErrorCode::NotFound), now: 1 };
    let (state, _) = run([Event::ChooseHome(native("/d/gone.md")), Event::Launch(None), missing]);
    assert!(matches!(state.status, Status::Failed(ErrorCode::NotFound)), "{:?}", state.status);
    let (state, effects) = step(state, Event::Enter(native("/d/a.md")));
    assert_eq!(state.address(), native("/d/a.md"));
    assert_eq!(effects.len(), 1);
}

#[test]
fn back_returns_from_home_and_reload_reloads_the_welcome_page() {
    let (state, _) = run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "x"), Event::Home]);
    assert!(state.nav.can_go_back());
    let (state, effects) = step(state, Event::Reload);
    assert_eq!(effects, vec![Effect::Load { request_id: 3, location: Location::Welcome, origin: Origin::Trusted }]);
    let (state, _) = step(state, Event::Back);
    assert_eq!(state.address(), native("/d/a.md"));
}

#[test]
fn welcome_page_links_go_to_the_web_and_anchors_scroll() {
    let (state, _) = run([Event::Home, loaded_at(1, Location::Welcome, "# Welcome")]);
    let (state, effects) = step(state, Event::Follow("https://example.com/a.md".into()));
    assert_eq!(state.address(), "https://example.com/a.md");
    assert_eq!(
        load_origins(&effects),
        [Origin::Public],
        "the bundled page points at the web, not the reader's network"
    );
    let (state, _) = run([Event::Home, loaded_at(1, Location::Welcome, "# Welcome")]);
    let (state, effects) = step(state, Event::Follow("relative.md".into()));
    assert_eq!((state.notices.len(), effects), (1, vec![]));
}

fn load_origins(effects: &[Effect]) -> Vec<Origin> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Load { origin, .. } => Some(*origin),
            _ => None,
        })
        .collect()
}

#[test]
fn typed_and_reopened_addresses_are_trusted() {
    let (_, effects) = run([Event::Enter("https://x.org/a.md".into())]);
    assert_eq!(load_origins(&effects), [Origin::Trusted]);
    let (_, effects) = run([Event::OpenHistory("https://x.org/a.md".into())]);
    assert_eq!(load_origins(&effects), [Origin::Trusted]);
}

#[test]
fn links_from_a_public_page_are_public_and_keep_it_on_reload_and_back() {
    let opened = [Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "# A")];
    let (state, effects) = run(opened.clone().into_iter().chain([Event::Follow("b.md".into())]));
    assert_eq!(load_origins(&effects), [Origin::Public]);
    let (state, effects) = step(state, Event::Reload);
    assert_eq!(load_origins(&effects), [Origin::Public]);
    let (state, _) = step(state, loaded_at(3, web("https://x.org/b.md"), "# B"));
    let (state, effects) = step(state, Event::Back);
    assert_eq!(load_origins(&effects), [Origin::Trusted], "a.md was typed");
    let (_, effects) = step(state, Event::Forward);
    assert_eq!(load_origins(&effects), [Origin::Public]);
}

#[test]
fn links_from_local_files_and_local_pages_are_trusted() {
    let (_, effects) =
        run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "# A"), Event::Follow("https://x.org/b.md".into())]);
    assert_eq!(load_origins(&effects), [Origin::Trusted]);
    let (_, effects) = run([
        Event::Enter("http://127.0.0.1:8000/a.md".into()),
        fetched(1, "http://127.0.0.1:8000/a.md", "# A"),
        Event::Follow("https://x.org/b.md".into()),
    ]);
    assert_eq!(load_origins(&effects), [Origin::Trusted]);
}

fn image_loads(effects: &[Effect]) -> Vec<(Location, Origin)> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::LoadImage { location, origin } => Some((location.clone(), *origin)),
            _ => None,
        })
        .collect()
}

fn picture() -> crate::session::images::Picture {
    let handle = iced::advanced::image::Handle::from_rgba(1, 1, vec![0; 4]);
    crate::session::images::Picture { width: 1, height: 1, handle }
}

#[test]
fn a_ready_page_loads_its_images_with_its_origin() {
    let (_, effects) =
        run([Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "![x](x.png)")]);
    assert_eq!(image_loads(&effects), [(web("https://x.org/x.png"), Origin::Public)]);
    let (_, effects) = run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "![x](x.png)")]);
    assert_eq!(image_loads(&effects), [(Location::file(native("/d/x.png")), Origin::Trusted)]);
}

#[test]
fn a_finished_image_starts_the_next_and_shows() {
    let source = "![1](1.png) ![2](2.png) ![3](3.png) ![4](4.png) ![5](5.png)";
    let (state, effects) = run([Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", source)]);
    assert_eq!(image_loads(&effects).len(), 4);
    let done = Event::ImageLoaded { location: web("https://x.org/1.png"), result: Ok(picture()) };
    let (state, effects) = step(state, done);
    assert_eq!(image_loads(&effects), [(web("https://x.org/5.png"), Origin::Public)]);
    assert_eq!(state.images.view("1.png"), View::Shown { width: 1, height: 1 });
}

#[test]
fn leaving_a_page_drops_its_waiting_loads() {
    let source = "![1](1.png) ![2](2.png) ![3](3.png) ![4](4.png) ![5](5.png)";
    let (state, _) = run([
        Event::Enter("https://x.org/a.md".into()),
        fetched(1, "https://x.org/a.md", source),
        Event::Enter("https://x.org/b.md".into()),
    ]);
    let done = Event::ImageLoaded { location: web("https://x.org/1.png"), result: Ok(picture()) };
    let (_, effects) = step(state, done);
    assert!(image_loads(&effects).is_empty());
}

fn opened_with(mode: ImageLoading, source: &str) -> State {
    let (state, _) = run([
        Event::ChooseImageLoading(mode),
        Event::Enter("https://x.org/a.md".into()),
        fetched(1, "https://x.org/a.md", source),
    ]);
    state
}

fn public(path: &str) -> (Location, Origin) {
    (web(&format!("https://x.org/{path}")), Origin::Public)
}

#[test]
fn off_loads_nothing_until_the_shortcut_which_keeps_the_setting() {
    let state = opened_with(ImageLoading::Off, "![x](x.png) ![y](y.png)");
    let (state, effects) = step(state, Event::LoadPageImages);
    assert_eq!(image_loads(&effects), [public("x.png"), public("y.png")]);
    assert_eq!(state.settings.images, ImageLoading::Off);
}

#[test]
fn switching_to_automatic_loads_the_page_and_saves_the_setting() {
    let state = opened_with(ImageLoading::Off, "![x](x.png) ![y](y.png)");
    let (_, effects) = step(state, Event::ChooseImageLoading(ImageLoading::Automatic));
    assert_eq!(image_loads(&effects), [public("x.png"), public("y.png")]);
    let saved = Settings { images: ImageLoading::Automatic, ..Settings::default() };
    assert!(effects.contains(&Effect::PersistSettings(saved)));
}

#[test]
fn a_click_loads_one_waiting_image_and_off_ignores_clicks() {
    let state = opened_with(ImageLoading::OnClick, "![x](x.png) ![y](y.png)");
    let (_, effects) = step(state, Event::LoadImage("x.png".into()));
    assert_eq!(image_loads(&effects), [public("x.png")]);
    let state = opened_with(ImageLoading::Off, "![x](x.png)");
    let (_, effects) = step(state, Event::LoadImage("x.png".into()));
    assert!(image_loads(&effects).is_empty());
}

#[test]
fn a_click_on_a_source_not_on_the_page_does_nothing() {
    let state = opened_with(ImageLoading::OnClick, "![x](x.png)");
    let (after, effects) = step(state.clone(), Event::LoadImage("other.png".into()));
    assert!(effects.is_empty());
    assert_eq!(after, state);
}

#[test]
fn switching_off_cancels_nothing_that_is_running() {
    let state = opened_with(ImageLoading::Automatic, "![x](x.png)");
    let (state, effects) = step(state, Event::ChooseImageLoading(ImageLoading::Off));
    assert!(image_loads(&effects).is_empty());
    let done = Event::ImageLoaded { location: web("https://x.org/x.png"), result: Ok(picture()) };
    let (state, _) = step(state, done);
    assert_eq!(state.images.view("x.png"), View::Shown { width: 1, height: 1 });
}

#[test]
fn a_failed_image_shows_its_note_and_refused_sources_load_nothing() {
    let state = opened_with(ImageLoading::Automatic, "![x](x.png) ![f](file:///etc/x.png)");
    let failed = Event::ImageLoaded { location: web("https://x.org/x.png"), result: Err(Problem::Unavailable) };
    let (state, effects) = step(state, failed);
    assert!(image_loads(&effects).is_empty());
    assert_eq!(state.images.view("x.png"), View::Note { note: Note::Failed, clickable: true });
    assert_eq!(state.images.view("file:///etc/x.png"), View::Note { note: Note::Refused, clickable: false });
}

#[test]
fn a_local_file_loads_web_images_as_the_readers_own() {
    let (_, effects) = run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "![w](https://x.org/w.png)")]);
    assert_eq!(image_loads(&effects), [(web("https://x.org/w.png"), Origin::Trusted)]);
}

#[test]
fn a_markdown_version_found_instead_becomes_the_address_and_the_history_entry() {
    let (state, effects) = run([
        Event::Enter("https://x.org/post".into()),
        found_instead(1, "https://x.org/post", "https://x.org/post.md", "# Post"),
    ]);
    assert_eq!(state.address(), "https://x.org/post.md");
    assert_eq!(history(&effects), ["https://x.org/post.md"]);
    assert_eq!(state.visits.len(), 1);
    assert_eq!(shown(&state).fallback_from.as_deref(), Some("https://x.org/post"));
}

#[test]
fn the_system_browser_opens_the_original_address_of_a_fallback() {
    let (state, _) = run([
        Event::Enter("https://x.org/post".into()),
        found_instead(1, "https://x.org/post", "https://x.org/post.md", "# Post"),
    ]);
    assert_eq!(state.browser_offer().as_deref(), Some("https://x.org/post"));
    let (_, effects) = step(state, Event::OpenInBrowser);
    assert_eq!(effects, [Effect::OpenExternal("https://x.org/post".into())]);
}

#[test]
fn a_page_that_was_markdown_all_along_carries_no_fallback() {
    let (state, _) = run([Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "# A")]);
    assert_eq!(shown(&state).fallback_from, None);
}

#[test]
fn going_back_to_a_fallback_page_shows_the_markdown_address_without_the_banner() {
    let (state, _) = run([
        Event::Enter("https://x.org/post".into()),
        found_instead(1, "https://x.org/post", "https://x.org/post.md", "# Post"),
        Event::Enter("https://x.org/b.md".into()),
        fetched(2, "https://x.org/b.md", "# B"),
        Event::Back,
        fetched(3, "https://x.org/post.md", "# Post"),
    ]);
    assert_eq!(state.address(), "https://x.org/post.md");
    assert_eq!(shown(&state).fallback_from, None);
}

#[test]
fn a_page_with_no_markdown_version_offers_the_sites_llms_txt() {
    let (state, _) =
        run([Event::Enter("https://x.org/a/b?q=1#top".into()), failed_with(ErrorCode::NotMarkdownWithLlms)]);
    assert_eq!(state.browser_offer().as_deref(), Some("https://x.org/a/b?q=1"));
    assert_eq!(state.llms_offer().as_deref(), Some("https://x.org/llms.txt"));
    let (state, _) = step(state, Event::OpenLlmsTxt);
    assert_eq!(state.status, Status::Loading);
    assert_eq!(state.address(), "https://x.org/llms.txt");
}

#[test]
fn opening_llms_txt_keeps_the_origin_of_the_failed_page() {
    let (state, _) = run([
        Event::Enter("https://x.org/a.md".into()),
        fetched(1, "https://x.org/a.md", "# A"),
        Event::Follow("https://y.org/post.md".into()),
        Event::Loaded { request_id: 2, result: Err(ErrorCode::NotMarkdownWithLlms), now: 1 },
    ]);
    let (_, effects) = step(state, Event::OpenLlmsTxt);
    assert!(
        matches!(&effects[..], [Effect::Load { origin: Origin::Public, location, .. }] if location.address() == "https://y.org/llms.txt"),
        "{effects:?}"
    );
}

#[test]
fn opening_llms_txt_with_no_offer_does_nothing() {
    let (state, _) = run([Event::Enter("https://x.org/a".into()), failed_with(ErrorCode::NotMarkdown)]);
    let (after, effects) = step(state.clone(), Event::OpenLlmsTxt);
    assert_eq!((after, effects), (state, vec![]));
}

#[test]
fn the_offer_keeps_a_port_and_ends_when_a_page_loads() {
    let (state, _) = run([Event::Enter("https://x.org:8443/a".into()), failed_with(ErrorCode::NotMarkdownWithLlms)]);
    assert_eq!(state.llms_offer().as_deref(), Some("https://x.org:8443/llms.txt"));
    let (state, _) = run([
        Event::Enter("https://x.org/a".into()),
        failed_with(ErrorCode::NotMarkdownWithLlms),
        Event::Enter("https://x.org/b.md".into()),
        fetched(2, "https://x.org/b.md", "# B"),
    ]);
    assert_eq!(state.llms_offer(), None);
}

#[test]
fn a_fallback_banner_does_not_outlive_the_next_page() {
    let (state, _) = run([
        Event::Enter("https://x.org/post".into()),
        found_instead(1, "https://x.org/post", "https://x.org/post.md", "# Post"),
        Event::Enter("https://x.org/b.md".into()),
        fetched(2, "https://x.org/b.md", "# B"),
    ]);
    assert_eq!(shown(&state).fallback_from, None);
    assert_eq!(state.browser_offer().as_deref(), None);
}

#[test]
fn llms_txt_is_offered_for_nothing_else() {
    let cases = [
        vec![Event::Enter("https://x.org/a".into()), failed_with(ErrorCode::NotFound)],
        vec![Event::Enter(native("/d/a.md")), failed_with(ErrorCode::NotMarkdown)],
        vec![Event::Enter("https://x.org/a.md".into()), fetched(1, "https://x.org/a.md", "# A")],
        vec![],
    ];
    for events in cases {
        let (state, _) = run(events.clone());
        assert_eq!(state.llms_offer(), None, "{events:?}");
    }
}

#[test]
fn a_post_names_its_neighbours() {
    let (state, _) = run([
        Event::Enter(native("/d/two.md")),
        loaded(1, "/d/two.md", "---\nurlPrev: one.md\nurlNext: ../three.md\n---\n# Two"),
    ]);
    assert_eq!(state.neighbour(Neighbour::Previous).as_deref(), Some("one.md"));
    assert_eq!(state.neighbour(Neighbour::Next).as_deref(), Some("../three.md"));
}

#[test]
fn no_neighbours_without_front_matter_or_a_page() {
    let (state, _) = run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "# A")]);
    assert_eq!((state.neighbour(Neighbour::Previous), state.neighbour(Neighbour::Next)), (None, None));
    let (loading, _) = run([Event::Enter(native("/d/a.md"))]);
    assert_eq!(loading.neighbour(Neighbour::Next), None);
}

#[test]
fn following_a_neighbour_is_an_ordinary_navigation() {
    let (state, effects) = run([
        Event::Enter(native("/d/two.md")),
        loaded(1, "/d/two.md", "---\nurlNext: three.md\n---\n# Two"),
        Event::Neighbour(Neighbour::Next),
    ]);
    assert_eq!(
        effects,
        [Effect::Load { request_id: 2, location: Location::file(native("/d/three.md")), origin: Origin::Trusted }]
    );
    assert_eq!(state.status, Status::Loading);
}

#[test]
fn a_neighbour_that_is_not_there_does_nothing() {
    let (state, effects) =
        run([Event::Enter(native("/d/a.md")), loaded(1, "/d/a.md", "# A"), Event::Neighbour(Neighbour::Next)]);
    assert!(effects.iter().all(|effect| !matches!(effect, Effect::Load { .. })), "{effects:?}");
    assert!(matches!(state.status, Status::Ready(_)));
}
