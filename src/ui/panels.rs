//! Which side panels are open, and the rule that keeps the page readable beside them.

/// Width of each side panel.
pub const PANEL_WIDTH: f32 = 300.0;
/// The page column never gets narrower than this because of a panel opening.
pub const MIN_PAGE: f32 = 320.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    History,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Open {
    pub history: bool,
    pub settings: bool,
}

impl Open {
    pub const fn has(self, panel: Panel) -> bool {
        match panel {
            Panel::History => self.history,
            Panel::Settings => self.settings,
        }
    }

    const fn with(self, panel: Panel, open: bool) -> Self {
        match panel {
            Panel::History => Self { history: open, ..self },
            Panel::Settings => Self { settings: open, ..self },
        }
    }
}

impl Open {
    /// The panels after Esc: Settings closes first, then History; none open stays none.
    pub const fn after_escape(self) -> Self {
        if self.settings { Self { settings: false, ..self } } else { Self { history: false, ..self } }
    }

    /// Whether `after` has closed a panel that this had open, so focus inside it was lost.
    pub const fn lost_a_panel(self, after: Self) -> bool {
        (self.history && !after.history) || (self.settings && !after.settings)
    }
}

const fn other(panel: Panel) -> Panel {
    match panel {
        Panel::History => Panel::Settings,
        Panel::Settings => Panel::History,
    }
}

/// The panels after the reader toggles `panel` in a window `width` wide. Opening one when both
/// would leave the page under `MIN_PAGE` closes the other, the older. A single panel is always
/// allowed, and a resize never calls this, so it never closes anything.
pub fn toggled(open: Open, panel: Panel, width: f32) -> Open {
    if open.has(panel) {
        return open.with(panel, false);
    }
    let opened = open.with(panel, true);
    let both_fit = width - 2.0 * PANEL_WIDTH >= MIN_PAGE;
    if opened.has(other(panel)) && !both_fit { opened.with(other(panel), false) } else { opened }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOMY: f32 = 1000.0;
    const NARROW: f32 = 700.0;

    #[test]
    fn a_panel_opens_and_closes() {
        let open = toggled(Open::default(), Panel::History, NARROW);
        assert_eq!(open, Open { history: true, settings: false });
        assert_eq!(toggled(open, Panel::History, NARROW), Open::default());
    }

    #[test]
    fn both_panels_stay_open_when_the_page_still_fits() {
        let open = toggled(Open { history: true, settings: false }, Panel::Settings, ROOMY);
        assert_eq!(open, Open { history: true, settings: true });
    }

    #[test]
    fn opening_a_second_panel_closes_the_older_one_when_the_page_would_be_squeezed() {
        let open = toggled(Open { history: true, settings: false }, Panel::Settings, NARROW);
        assert_eq!(open, Open { history: false, settings: true });
        let open = toggled(open, Panel::History, NARROW);
        assert_eq!(open, Open { history: true, settings: false });
    }

    #[test]
    fn the_page_may_be_exactly_the_minimum() {
        let width = 2.0 * PANEL_WIDTH + MIN_PAGE;
        let both = Open { history: true, settings: true };
        assert_eq!(toggled(Open { history: true, settings: false }, Panel::Settings, width), both);
        assert_ne!(toggled(Open { history: true, settings: false }, Panel::Settings, width - 1.0), both);
    }

    #[test]
    fn a_single_panel_is_allowed_in_the_smallest_window() {
        let open = toggled(Open::default(), Panel::Settings, 480.0);
        assert!(open.settings);
    }

    #[test]
    fn escape_closes_settings_first_then_history_and_leaves_none_alone() {
        let both = Open { history: true, settings: true };
        assert_eq!(both.after_escape(), Open { history: true, settings: false });
        assert_eq!(both.after_escape().after_escape(), Open::default());
        assert_eq!(Open { history: false, settings: true }.after_escape(), Open::default());
        assert_eq!(Open::default().after_escape(), Open::default());
    }

    #[test]
    fn a_panel_is_lost_when_one_that_was_open_is_closed() {
        let both = Open { history: true, settings: true };
        let history = Open { history: true, settings: false };
        assert!(both.lost_a_panel(history));
        assert!(history.lost_a_panel(Open::default()));
        assert!(history.lost_a_panel(Open { history: false, settings: true }), "swapped: History went");
        assert!(!history.lost_a_panel(both), "opening one loses none");
        assert!(!Open::default().lost_a_panel(history));
    }

    #[test]
    fn closing_one_of_two_leaves_the_other() {
        let both = Open { history: true, settings: true };
        assert_eq!(toggled(both, Panel::History, NARROW), Open { history: false, settings: true });
    }
}
