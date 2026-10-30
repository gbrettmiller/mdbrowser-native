//! Tests for the image planner.

use std::path::Path;

use iced::advanced::image::Handle;
use url::Url;

use super::*;
use crate::session::images::limits::MAX_CONCURRENT_LOADS;
use crate::session::images::{ImageLoading, Note, Problem};

/// Checks of the planner's own state, which only tests need.
impl Images {
    /// Bytes of every shown picture; only eviction makes it fall.
    const fn shown_bytes(&self) -> u64 {
        self.shown_bytes
    }

    /// Checks the planner's own consistency, for tests: the queue holds exactly the
    /// queued entries, waiting/off/queued entries belong to the current page, the
    /// running totals match the table, the cache tracks only entries, and a
    /// non-empty queue means every load slot is in use.
    fn check(&self) {
        let queued: HashSet<&Location> = self
            .table
            .iter()
            .filter(|(_, entry)| matches!(entry, Entry::Queued))
            .map(|(location, _)| location)
            .collect();
        assert_eq!(queued, self.queue.iter().collect(), "queue and queued entries agree");
        assert_eq!(queued.len(), self.queue.len(), "nothing is queued twice");
        for (location, entry) in &self.table {
            if matches!(entry, Entry::Queued | Entry::AwaitingClick | Entry::Off) {
                assert!(self.pinned.contains(location), "{location:?} is {entry:?} but not on the page");
            }
        }
        let loading = self.table.values().filter(|entry| matches!(entry, Entry::Loading)).count();
        assert_eq!(self.loading, loading, "the running count matches the table");
        let shown: u64 = self.shown().map(|(_, picture)| picture.pixel_bytes()).sum();
        assert_eq!(self.shown_bytes, shown, "the shown total matches the table");
        assert!(
            self.cache.locations().all(|location| self.table.contains_key(location)),
            "the cache tracks only entries"
        );
        assert!(self.queue.is_empty() || self.loading == MAX_CONCURRENT_LOADS, "a queue never waits for a free slot");
    }
}

fn page() -> Location {
    Location::web(Url::parse("https://x.org/post.md").unwrap())
}

fn file_page() -> Location {
    Location::file("/d/post.md")
}

fn at(path: &str) -> Location {
    Location::web(Url::parse(&format!("https://x.org/{path}")).unwrap())
}

fn picture(width: u32, height: u32) -> Picture {
    Picture { width, height, handle: Handle::from_rgba(width, height, vec![0; (width * height * 4) as usize]) }
}

fn srcs(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn open(images: Images, names: &[&str], mode: ImageLoading) -> (Images, Vec<Location>) {
    images.open_page(&srcs(names), &page(), Path::new("/home/r"), mode)
}

fn view(images: &Images, src: &str) -> View {
    images.view(src)
}

#[test]
fn a_page_starts_four_loads_and_each_finished_one_starts_the_next() {
    let names = ["1.png", "2.png", "3.png", "4.png", "5.png", "6.png"];
    let (images, loads) = open(Images::new(), &names, ImageLoading::Automatic);
    assert_eq!(loads.len(), MAX_CONCURRENT_LOADS);
    assert_eq!(loads, ["1.png", "2.png", "3.png", "4.png"].map(at));
    assert_eq!(view(&images, "5.png"), View::Note { note: Note::Loading, clickable: false });
    let (images, next) = images.loaded(at("1.png"), Ok(picture(2, 2)));
    assert_eq!(next, [at("5.png")]);
    assert_eq!(view(&images, "1.png"), View::Shown { width: 2, height: 2 });
    let (_, next) = images.loaded(at("2.png"), Err(Problem::Unavailable));
    assert_eq!(next, [at("6.png")]);
}

#[test]
fn a_repeated_image_loads_once() {
    let (_, loads) = open(Images::new(), &["a.png", "a.png", "./a.png"], ImageLoading::Automatic);
    assert_eq!(loads, [at("a.png")]);
}

#[test]
fn the_cap_holds_across_pages_and_leaving_drops_the_queue() {
    let names: Vec<String> = (1..=6).map(|i| format!("{i}.png")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let (images, _) = open(Images::new(), &names, ImageLoading::Automatic);
    let images = images.leave_page();
    let (images, loads) = open(images, &["b.png"], ImageLoading::Automatic);
    assert!(loads.is_empty(), "four are still running");
    let (images, next) = images.loaded(at("1.png"), Ok(picture(1, 1)));
    assert_eq!(next, [at("b.png")], "the old page's queue was dropped");
    assert_eq!(images.loading_count(), 4);
}

#[test]
fn a_result_for_another_page_is_cached_but_changes_nothing_on_screen() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::Automatic);
    let images = images.leave_page();
    let (images, _) = open(images, &["b.png"], ImageLoading::Automatic);
    let generation = images.generation();
    let (images, _) = images.loaded(at("a.png"), Ok(picture(1, 1)));
    assert_eq!(images.generation(), generation);
    let images = images.leave_page();
    let (images, loads) = open(images, &["a.png"], ImageLoading::Automatic);
    assert!(loads.is_empty(), "cached");
    assert_eq!(view(&images, "a.png"), View::Shown { width: 1, height: 1 });
}

#[test]
fn a_result_for_this_page_bumps_the_generation() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::Automatic);
    let generation = images.generation();
    let (images, _) = images.loaded(at("a.png"), Ok(picture(1, 1)));
    assert!(images.generation() > generation);
}

#[test]
fn on_click_waits_and_a_click_loads_once() {
    let (images, loads) = open(Images::new(), &["a.png"], ImageLoading::OnClick);
    assert!(loads.is_empty());
    assert_eq!(view(&images, "a.png"), View::Note { note: Note::ClickToLoad, clickable: true });
    let (images, loads) = images.click(&at("a.png"));
    assert_eq!(loads, [at("a.png")]);
    let (_, loads) = images.click(&at("a.png"));
    assert!(loads.is_empty(), "already loading");
}

#[test]
fn off_requests_nothing_and_ignores_clicks() {
    let (images, loads) = open(Images::new(), &["a.png"], ImageLoading::Off);
    assert!(loads.is_empty());
    assert_eq!(view(&images, "a.png"), View::Note { note: Note::Off, clickable: false });
    let (_, loads) = images.click(&at("a.png"));
    assert!(loads.is_empty());
}

#[test]
fn local_images_load_in_every_mode() {
    for mode in ImageLoading::ALL {
        let (_, loads) = Images::new().open_page(&srcs(&["pic.png"]), &file_page(), Path::new("/h"), mode);
        assert_eq!(loads, [Location::file("/d/pic.png")], "{mode:?}");
    }
}

#[test]
fn a_click_retries_a_failed_image_and_revisiting_retries_it_too() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::Automatic);
    let (images, _) = images.loaded(at("a.png"), Err(Problem::Unavailable));
    assert_eq!(view(&images, "a.png"), View::Note { note: Note::Failed, clickable: true });
    let (images, loads) = images.click(&at("a.png"));
    assert_eq!(loads, [at("a.png")]);
    let (images, _) = images.loaded(at("a.png"), Err(Problem::Unsupported));
    assert_eq!(view(&images, "a.png"), View::Note { note: Note::Unsupported, clickable: false });
    let images = images.leave_page();
    let (_, loads) = open(images, &["a.png"], ImageLoading::Automatic);
    assert_eq!(loads, [at("a.png")], "failures aren't cached");
}

#[test]
fn refused_sources_show_their_note_and_load_nothing() {
    let (images, loads) = open(Images::new(), &["file:///etc/x.png", ""], ImageLoading::Automatic);
    assert!(loads.is_empty());
    assert_eq!(view(&images, "file:///etc/x.png"), View::Note { note: Note::Refused, clickable: false });
}

#[test]
fn switching_modes_never_hides_or_cancels_and_moves_the_rest() {
    // automatic → off: queued become off; shown and loading stay.
    let names = ["1.png", "2.png", "3.png", "4.png", "5.png", "6.png"];
    let (images, _) = open(Images::new(), &names, ImageLoading::Automatic);
    let (images, started) = images.loaded(at("1.png"), Ok(picture(1, 1)));
    assert_eq!(started, [at("5.png")]);
    let (images, loads) = images.mode_changed(ImageLoading::Off);
    assert!(loads.is_empty());
    assert_eq!(view(&images, "1.png"), View::Shown { width: 1, height: 1 });
    assert_eq!(view(&images, "5.png"), View::Note { note: Note::Loading, clickable: false }, "still running");
    assert_eq!(view(&images, "6.png"), View::Note { note: Note::Off, clickable: false });
    // off → on click: off become waiting.
    let (images, loads) = images.mode_changed(ImageLoading::OnClick);
    assert!(loads.is_empty());
    assert_eq!(view(&images, "6.png"), View::Note { note: Note::ClickToLoad, clickable: true });
    // on click → off: waiting become off.
    let (images, _) = images.mode_changed(ImageLoading::Off);
    assert_eq!(view(&images, "6.png"), View::Note { note: Note::Off, clickable: false });
    // off → automatic: off are queued and start when there is room.
    let (images, loads) = images.mode_changed(ImageLoading::Automatic);
    assert!(loads.is_empty(), "four are still running");
    let (_, next) = images.loaded(at("2.png"), Ok(picture(1, 1)));
    assert_eq!(next, [at("6.png")]);
}

#[test]
fn automatic_to_on_click_turns_queued_images_into_waiting_ones() {
    let names = ["1.png", "2.png", "3.png", "4.png", "5.png"];
    let (images, _) = open(Images::new(), &names, ImageLoading::Automatic);
    let (images, _) = images.mode_changed(ImageLoading::OnClick);
    assert_eq!(view(&images, "5.png"), View::Note { note: Note::ClickToLoad, clickable: true });
    let (images, loads) = images.mode_changed(ImageLoading::Automatic);
    assert!(loads.is_empty());
    let (_, next) = images.loaded(at("1.png"), Ok(picture(1, 1)));
    assert_eq!(next, [at("5.png")]);
}

#[test]
fn loading_this_pages_images_loads_waiting_off_and_failed_ones() {
    let (images, _) = open(Images::new(), &["a.png", "b.png"], ImageLoading::Off);
    let (images, loads) = images.load_page();
    assert_eq!(loads, [at("a.png"), at("b.png")]);
    let (images, _) = images.loaded(at("a.png"), Err(Problem::Unavailable));
    let (_, loads) = images.load_page();
    assert_eq!(loads, [at("a.png")]);
}

#[test]
fn the_cache_evicts_the_least_recently_used_but_never_this_pages_images() {
    let budget = picture(10, 10).pixel_bytes() * 2;
    let mut images = Images::with_budget(budget);
    for name in ["a.png", "b.png", "c.png"] {
        let (opened, _) = open(images.leave_page(), &[name], ImageLoading::Automatic);
        let (loaded, _) = opened.loaded(at(name), Ok(picture(10, 10)));
        images = loaded;
    }
    // a.png is the least recently used; c.png is on the current page.
    let (images, loads) = open(images.leave_page(), &["a.png"], ImageLoading::Automatic);
    assert_eq!(loads, [at("a.png")], "evicted, so fetched again");
    let (images, _) = images.loaded(at("a.png"), Ok(picture(10, 10)));
    let (images, loads) = open(images.leave_page(), &["c.png"], ImageLoading::Automatic);
    assert!(loads.is_empty(), "c.png was used more recently than b.png");
    let (_, loads) = open(images.leave_page(), &["b.png"], ImageLoading::Automatic);
    assert_eq!(loads, [at("b.png")], "b.png was the one dropped");
}

#[test]
fn pinned_images_may_exceed_the_budget_while_shown() {
    let images = Images::with_budget(picture(10, 10).pixel_bytes());
    let (images, _) = open(images, &["a.png", "b.png"], ImageLoading::Automatic);
    let (images, _) = images.loaded(at("a.png"), Ok(picture(10, 10)));
    let (images, _) = images.loaded(at("b.png"), Ok(picture(10, 10)));
    assert_eq!(view(&images, "a.png"), View::Shown { width: 10, height: 10 });
    assert_eq!(view(&images, "b.png"), View::Shown { width: 10, height: 10 });
    let images = images.leave_page();
    assert!(images.unpinned_bytes() <= picture(10, 10).pixel_bytes(), "leaving evicts back to budget");
}

#[test]
fn a_click_off_the_current_page_does_nothing() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::OnClick);
    let generation = images.generation();
    let (images, loads) = images.click(&at("elsewhere.png"));
    assert!(loads.is_empty());
    assert_eq!(images.generation(), generation);
    let (_, loads) = images.leave_page().click(&at("a.png"));
    assert!(loads.is_empty(), "a.png's page was left");
}

#[test]
fn only_images_that_failed_to_load_can_be_retried() {
    let (images, _) = open(Images::new(), &["a.png", "b.png"], ImageLoading::Automatic);
    let (images, _) = images.loaded(at("a.png"), Err(Problem::Unsupported));
    let (images, _) = images.loaded(at("b.png"), Err(Problem::TooLarge));
    let (images, loads) = images.click(&at("a.png"));
    assert!(loads.is_empty());
    let (_, loads) = images.load_page();
    assert!(loads.is_empty());
}

#[test]
fn loading_this_pages_images_leaves_local_ones_alone() {
    let srcs = srcs(&["local.png", "https://e.com/web.png"]);
    let (images, _) = Images::new().open_page(&srcs, &file_page(), Path::new("/h"), ImageLoading::Off);
    let (images, _) = images.loaded(Location::file("/d/local.png"), Err(Problem::Unavailable));
    let (_, loads) = images.load_page();
    assert_eq!(loads, [Location::web(Url::parse("https://e.com/web.png").unwrap())]);
}

#[test]
fn a_failure_for_another_page_is_forgotten_so_a_revisit_retries() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::Automatic);
    let (images, _) = open(images.leave_page(), &["b.png"], ImageLoading::Automatic);
    let generation = images.generation();
    let (images, _) = images.loaded(at("a.png"), Err(Problem::Unavailable));
    assert_eq!(images.generation(), generation);
    let (_, loads) = open(images.leave_page(), &["a.png"], ImageLoading::Automatic);
    assert_eq!(loads, [at("a.png")]);
}

#[test]
fn a_load_shared_by_two_pages_runs_once_and_lands_on_the_second() {
    let (images, _) = open(Images::new(), &["x.png"], ImageLoading::Automatic);
    let (images, loads) = open(images.leave_page(), &["x.png"], ImageLoading::Automatic);
    assert!(loads.is_empty(), "still running from the first page");
    assert_eq!(images.loading_count(), 1);
    let generation = images.generation();
    let (images, _) = images.loaded(at("x.png"), Err(Problem::Unavailable));
    assert!(images.generation() > generation);
    assert_eq!(view(&images, "x.png"), View::Note { note: Note::Failed, clickable: true });
}

#[test]
fn a_result_for_a_load_never_started_is_ignored() {
    let (images, _) = open(Images::new(), &["a.png"], ImageLoading::OnClick);
    let (images, loads) = images.loaded(at("a.png"), Ok(picture(1, 1)));
    assert!(loads.is_empty());
    assert_eq!(view(&images, "a.png"), View::Note { note: Note::ClickToLoad, clickable: true });
    let (images, _) = images.loaded(at("never.png"), Ok(picture(1, 1)));
    assert_eq!(images.loading_count(), 0);
    images.check();
}

#[test]
fn reopening_a_page_mid_load_starts_nothing_twice() {
    let names = ["1.png", "2.png", "3.png", "4.png", "5.png", "6.png"];
    let (images, _) = open(Images::new(), &names, ImageLoading::Automatic);
    let (images, loads) = open(images.leave_page(), &names, ImageLoading::Automatic);
    assert!(loads.is_empty(), "four still running, two queued again");
    images.check();
    let (images, next) = images.loaded(at("1.png"), Ok(picture(1, 1)));
    assert_eq!(next, [at("5.png")]);
    let (_, next) = images.loaded(at("2.png"), Ok(picture(1, 1)));
    assert_eq!(next, [at("6.png")]);
}

/// Over random sequences of events, several seeds: the planner stays consistent
/// (`Images::check`), never runs more than `MAX_CONCURRENT_LOADS`, keeps unpinned pictures
/// within budget, and the run really reaches the cap, eviction and failures.
#[test]
fn invariants_hold_over_random_events() {
    let names: Vec<String> = (0..9).map(|i| format!("{i}.png")).collect();
    let budget = picture(4, 4).pixel_bytes() * 3;
    let (mut reached_cap, mut evicted, mut failed) = (false, false, false);
    for seed in [0x2b_u64, 7, 99, 2026, 31_337] {
        let mut seed = seed;
        let mut next = |n: u64| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % n
        };
        let mut images = Images::with_budget(budget);
        let mut page: Vec<String> = Vec::new();
        let mut running: Vec<Location> = Vec::new();
        for _ in 0..2_000 {
            let shown_before = images.shown_bytes();
            let (changed, started) = match next(7) {
                0 => {
                    let start = next(9) as usize;
                    let count = 3 + next(4) as usize; // 3 to 6 distinct sources
                    page = (0..count).map(|i| names[(start + i) % names.len()].clone()).collect();
                    let refs: Vec<&str> = page.iter().map(String::as_str).collect();
                    open(images.leave_page(), &refs, ImageLoading::ALL[next(3) as usize])
                }
                1 | 2 if !running.is_empty() => {
                    let location = running.remove(next(running.len() as u64) as usize);
                    let result = if next(3) == 0 { Err(Problem::Unavailable) } else { Ok(picture(4, 4)) };
                    failed |= result.is_err();
                    images.loaded(location, result)
                }
                3 if !page.is_empty() => images.click(&at(&page[next(page.len() as u64) as usize])),
                3 => images.click(&at(&names[next(9) as usize])),
                4 => images.load_page(),
                5 => images.mode_changed(ImageLoading::ALL[next(3) as usize]),
                _ => {
                    page.clear();
                    (images.leave_page(), Vec::new())
                }
            };
            evicted |= changed.shown_bytes() < shown_before;
            images = changed;
            running.extend(started);
            images.check();
            assert!(images.loading_count() <= MAX_CONCURRENT_LOADS);
            assert_eq!(images.loading_count(), running.len());
            assert!(images.unpinned_bytes() <= budget);
            reached_cap |= images.loading_count() == MAX_CONCURRENT_LOADS;
        }
    }
    assert!(reached_cap && evicted && failed, "cap {reached_cap}, eviction {evicted}, failures {failed}");
}

#[test]
fn the_cache_forgets_images_that_leave_the_table() {
    let (images, _) = open(Images::new(), &["a.png", "b.png"], ImageLoading::OnClick);
    let images = images.leave_page();
    assert_eq!(images.cache.tracked(), 0, "waiting images left with their page");
    let (images, _) = open(images, &["c.png"], ImageLoading::Automatic);
    let images = images.leave_page();
    let (images, _) = images.loaded(at("c.png"), Err(Problem::Unavailable));
    assert_eq!(images.cache.tracked(), 0, "a failure off the page is forgotten");
}
