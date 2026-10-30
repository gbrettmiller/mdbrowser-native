//! Keyboard stops on a page: its links and clickable image notes in reading order,
//! and the rule for moving between them.

use iced::Rectangle;

use super::Page;

/// What activating a stop does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Follow this link.
    Link(String),
    /// Load this image.
    Image(String),
}

/// One thing Tab can land on: a link or image note, with a rectangle per line it covers.
#[derive(Debug, Clone, PartialEq)]
pub struct Stop {
    pub target: Target,
    pub rects: Vec<Rectangle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Back,
}

/// Where a Tab goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// To this stop on the page.
    Stop(usize),
    /// Off the page, to the next or previous control.
    Leave,
}

/// Where `direction` takes focus from `current` (none: not on a stop yet) on a page of `count` stops.
pub fn reach(count: usize, current: Option<usize>, direction: Direction) -> Reach {
    let stop = match (direction, current) {
        (Direction::Forward, None) => Some(0),
        (Direction::Forward, Some(i)) => i.checked_add(1),
        (Direction::Back, None) => count.checked_sub(1),
        // A stop past the end (the page changed under focus) steps back from the last one.
        (Direction::Back, Some(i)) => i.min(count).checked_sub(1),
    };
    stop.filter(|&i| i < count).map_or(Reach::Leave, Reach::Stop)
}

impl Page {
    /// The links and image notes, in reading order. Fragments that follow one another with
    /// the same target are one stop, so a link that wraps or changes style has one.
    pub fn stops(&self) -> Vec<Stop> {
        let mut stops: Vec<Stop> = Vec::new();
        // The target and line of the fragment before, to tell a continuation from a new stop.
        let mut before: Option<(Target, usize)> = None;
        for fragment in &self.fragments {
            let target = match (&fragment.link, &fragment.action) {
                (Some(href), _) => Target::Link(href.clone()),
                (None, Some(src)) => Target::Image(src.clone()),
                (None, None) => {
                    before = None;
                    continue;
                }
            };
            match (&before, stops.last_mut()) {
                (Some((previous, line)), Some(stop)) if *previous == target => match stop.rects.last_mut() {
                    Some(rect) if *line == fragment.line => *rect = rect.union(&fragment.bounds),
                    _ => stop.rects.push(fragment.bounds),
                },
                _ => stops.push(Stop { target: target.clone(), rects: vec![fragment.bounds] }),
            }
            before = Some((target, fragment.line));
        }
        stops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tests::page_at;
    use crate::layout::{Fragment, Joint};
    use crate::theme::ReadingFont;
    use iced::{Color, Font};

    fn page(source: &str) -> Page {
        page_at(source, ReadingFont::Literata, 400.0)
    }

    fn targets(page: &Page) -> Vec<Target> {
        page.stops().into_iter().map(|stop| stop.target).collect()
    }

    fn link(href: &str) -> Target {
        Target::Link(href.into())
    }

    #[test]
    fn links_are_stops_in_reading_order_across_blocks() {
        let p = page(
            "[a](a.md) then text\n\n- [b](b.md)\n- plain\n- [c](c.md)\n\n> [d](d.md)\n\n| h |\n|---|\n| [e](e.md) |",
        );
        assert_eq!(targets(&p), [link("a.md"), link("b.md"), link("c.md"), link("d.md"), link("e.md")]);
    }

    #[test]
    fn a_page_without_links_has_no_stops() {
        assert!(page("Just words.\n\n# And a heading").stops().is_empty());
    }

    #[test]
    fn a_link_that_wraps_is_one_stop_with_a_rectangle_per_line() {
        let source = format!("[{}](long.md)", "word ".repeat(40));
        let p = page(&source);
        let stops = p.stops();
        assert_eq!(stops.len(), 1);
        assert!(stops[0].rects.len() > 2, "{} lines", stops[0].rects.len());
        let tops: Vec<f32> = stops[0].rects.iter().map(|r| r.y).collect();
        assert!(tops.windows(2).all(|w| w[0] < w[1]), "one rectangle per line, top down: {tops:?}");
    }

    #[test]
    fn a_link_that_changes_style_is_one_stop() {
        let p = page("[plain **bold** plain](a.md)");
        assert!(p.fragments.iter().filter(|f| f.link.is_some()).count() > 1);
        assert_eq!(targets(&p), [link("a.md")]);
    }

    #[test]
    fn two_links_to_the_same_address_are_two_stops_when_text_comes_between() {
        let p = page("[one](a.md) and [two](a.md)");
        assert_eq!(targets(&p), [link("a.md"), link("a.md")]);
    }

    fn fragment(link: Option<&str>, action: Option<&str>, y: f32) -> Fragment {
        Fragment {
            bounds: Rectangle { x: 0.0, y, width: 50.0, height: 20.0 },
            text: "x".into(),
            font: Font::DEFAULT,
            size: 18.0,
            color: Color::BLACK,
            link: link.map(Into::into),
            stops: vec![],
            line: (y / 20.0) as usize,
            joint: Joint::Break,
            block: 0,
            action: action.map(Into::into),
            note: false,
        }
    }

    fn built(fragments: Vec<Fragment>) -> Page {
        Page { texts: vec![], fills: vec![], fragments, images: vec![], anchors: vec![], height: 100.0 }
    }

    #[test]
    fn an_image_note_is_a_stop_that_loads_the_image() {
        let p = built(vec![
            fragment(None, None, 0.0),
            fragment(None, Some("a.png"), 20.0),
            fragment(Some("b.md"), None, 40.0),
        ]);
        assert_eq!(targets(&p), [Target::Image("a.png".into()), link("b.md")]);
    }

    #[test]
    fn a_stop_covers_the_fragments_it_is_made_of() {
        let p = built(vec![fragment(Some("a.md"), None, 0.0), fragment(Some("a.md"), None, 20.0)]);
        let rects = &p.stops()[0].rects;
        assert_eq!((rects[0].y, rects[1].y), (0.0, 20.0));
    }

    #[test]
    fn tab_walks_forward_and_shift_tab_back() {
        use Direction::{Back, Forward};
        assert_eq!(reach(3, None, Forward), Reach::Stop(0));
        assert_eq!(reach(3, Some(0), Forward), Reach::Stop(1));
        assert_eq!(reach(3, Some(1), Back), Reach::Stop(0));
        assert_eq!(reach(3, None, Back), Reach::Stop(2), "Shift+Tab into the page lands on the last link");
    }

    #[test]
    fn tab_leaves_the_page_past_either_end() {
        use Direction::{Back, Forward};
        assert_eq!(reach(3, Some(2), Forward), Reach::Leave);
        assert_eq!(reach(3, Some(0), Back), Reach::Leave);
    }

    #[test]
    fn a_page_without_stops_is_passed_straight_through() {
        assert_eq!(reach(0, None, Direction::Forward), Reach::Leave);
        assert_eq!(reach(0, None, Direction::Back), Reach::Leave);
    }

    #[test]
    fn a_stale_stop_past_the_end_leaves_rather_than_panics() {
        assert_eq!(reach(2, Some(5), Direction::Forward), Reach::Leave);
        assert_eq!(reach(2, Some(5), Direction::Back), Reach::Stop(1));
    }
}
