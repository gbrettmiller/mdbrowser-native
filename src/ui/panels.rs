//! Which side panels are open, and the rule that keeps the page readable beside them.
//!
//! Bookmarks and History are lists that share one side: opening one closes the other.
//! Settings has the other side.

/// Width of each side panel.
pub const PANEL_WIDTH: f32 = 300.0;
/// The page column never gets narrower than this because of a panel opening.
pub const MIN_PAGE: f32 = 320.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Bookmarks,
    History,
    Settings,
}

/// The list side's panel, if one is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    Bookmarks,
    History,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Open {
    pub list: Option<List>,
    pub settings: bool,
}

impl Open {
    pub const fn has(self, panel: Panel) -> bool {
        match panel {
            Panel::Bookmarks => matches!(self.list, Some(List::Bookmarks)),
            Panel::History => matches!(self.list, Some(List::History)),
            Panel::Settings => self.settings,
        }
    }

    /// The panels after Esc: Settings closes first, then the list; none open stays none.
    pub const fn after_escape(self) -> Self {
        if self.settings { Self { settings: false, ..self } } else { Self { list: None, ..self } }
    }

    /// Whether `after` has closed a panel that this had open, so focus inside it was lost.
    pub fn lost_a_panel(self, after: Self) -> bool {
        let list_went = self.list.is_some() && self.list != after.list;
        list_went || (self.settings && !after.settings)
    }

    const fn any(self) -> bool {
        self.list.is_some() || self.settings
    }
}

/// What Esc does: the panels afterwards, and whether the source view is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Escaped {
    pub panels: Open,
    pub leave_source: bool,
}

/// Esc closes a panel first, so the source view is left only when none is open.
pub const fn escaped(open: Open, in_source: bool) -> Escaped {
    let panels = open.after_escape();
    Escaped { panels, leave_source: in_source && !open.any() }
}

/// The panels after the reader toggles `panel` in a window `width` wide. A list replaces the
/// other list. Opening one side when both would leave the page under `MIN_PAGE` closes the
/// other side, the older. A single panel is always allowed, and a resize never calls this, so
/// it never closes anything.
pub fn toggled(open: Open, panel: Panel, width: f32) -> Open {
    let both_fit = width - 2.0 * PANEL_WIDTH >= MIN_PAGE;
    let list = |list| if open.list == Some(list) { None } else { Some(list) };
    match panel {
        Panel::Settings if open.settings => Open { settings: false, ..open },
        Panel::Settings => Open { settings: true, list: open.list.filter(|_| both_fit) },
        Panel::Bookmarks | Panel::History => {
            let list = list(if panel == Panel::Bookmarks { List::Bookmarks } else { List::History });
            Open { list, settings: open.settings && (list.is_none() || both_fit) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOMY: f32 = 1000.0;
    const NARROW: f32 = 700.0;
    const HISTORY: Open = Open { list: Some(List::History), settings: false };
    const BOOKMARKS: Open = Open { list: Some(List::Bookmarks), settings: false };
    const SETTINGS: Open = Open { list: None, settings: true };
    const BOTH: Open = Open { list: Some(List::History), settings: true };

    #[test]
    fn a_panel_opens_and_closes() {
        for (panel, alone) in [(Panel::History, HISTORY), (Panel::Bookmarks, BOOKMARKS), (Panel::Settings, SETTINGS)] {
            let open = toggled(Open::default(), panel, NARROW);
            assert_eq!(open, alone);
            assert!(open.has(panel));
            assert_eq!(toggled(open, panel, NARROW), Open::default());
        }
    }

    #[test]
    fn bookmarks_and_history_share_a_side_whatever_the_width() {
        assert_eq!(toggled(HISTORY, Panel::Bookmarks, ROOMY), BOOKMARKS);
        assert_eq!(toggled(BOOKMARKS, Panel::History, ROOMY), HISTORY);
        assert_eq!(toggled(BOTH, Panel::Bookmarks, ROOMY), Open { list: Some(List::Bookmarks), settings: true });
        assert_eq!(toggled(BOTH, Panel::Bookmarks, NARROW), Open { list: Some(List::Bookmarks), settings: false });
    }

    #[test]
    fn both_sides_stay_open_when_the_page_still_fits() {
        assert_eq!(toggled(HISTORY, Panel::Settings, ROOMY), BOTH);
        assert_eq!(toggled(SETTINGS, Panel::History, ROOMY), BOTH);
    }

    #[test]
    fn opening_a_second_side_closes_the_older_one_when_the_page_would_be_squeezed() {
        let open = toggled(HISTORY, Panel::Settings, NARROW);
        assert_eq!(open, SETTINGS);
        assert_eq!(toggled(open, Panel::History, NARROW), HISTORY);
        assert_eq!(toggled(SETTINGS, Panel::Bookmarks, NARROW), BOOKMARKS);
    }

    #[test]
    fn the_page_may_be_exactly_the_minimum() {
        let width = 2.0 * PANEL_WIDTH + MIN_PAGE;
        assert_eq!(toggled(HISTORY, Panel::Settings, width), BOTH);
        assert_ne!(toggled(HISTORY, Panel::Settings, width - 1.0), BOTH);
    }

    #[test]
    fn a_single_panel_leaves_the_page_its_minimum_in_the_smallest_window() {
        let (width, _) = crate::ui::app::MIN_WINDOW;
        assert!(toggled(Open::default(), Panel::Settings, width).settings);
        assert!(width - PANEL_WIDTH >= MIN_PAGE, "the page is {} px beside a panel", width - PANEL_WIDTH);
    }

    #[test]
    fn escape_closes_settings_first_then_the_list_and_leaves_none_alone() {
        assert_eq!(BOTH.after_escape(), HISTORY);
        assert_eq!(BOTH.after_escape().after_escape(), Open::default());
        assert_eq!(BOOKMARKS.after_escape(), Open::default());
        assert_eq!(SETTINGS.after_escape(), Open::default());
        assert_eq!(Open::default().after_escape(), Open::default());
    }

    #[test]
    fn escape_closes_a_panel_before_it_leaves_the_source_view() {
        let none = Open::default();
        for in_source in [false, true] {
            assert_eq!(escaped(SETTINGS, in_source), Escaped { panels: none, leave_source: false });
            assert_eq!(escaped(BOOKMARKS, in_source), Escaped { panels: none, leave_source: false });
            assert_eq!(escaped(BOTH, in_source), Escaped { panels: HISTORY, leave_source: false });
        }
        assert_eq!(escaped(none, true), Escaped { panels: none, leave_source: true });
        assert_eq!(escaped(none, false), Escaped { panels: none, leave_source: false });
    }

    #[test]
    fn a_panel_is_lost_when_one_that_was_open_is_closed_or_replaced() {
        assert!(BOTH.lost_a_panel(HISTORY));
        assert!(HISTORY.lost_a_panel(Open::default()));
        assert!(HISTORY.lost_a_panel(SETTINGS), "swapped: History went");
        assert!(HISTORY.lost_a_panel(BOOKMARKS), "Bookmarks took History's side");
        assert!(!HISTORY.lost_a_panel(BOTH), "opening one loses none");
        assert!(!Open::default().lost_a_panel(HISTORY));
    }

    #[test]
    fn closing_one_side_leaves_the_other() {
        assert_eq!(toggled(BOTH, Panel::History, NARROW), SETTINGS);
        assert_eq!(toggled(BOTH, Panel::Settings, NARROW), HISTORY);
    }
}
