//! The document widget: paints a laid-out `Page` (shaped once, at layout) and turns
//! pointer and keyboard input into link follows, hovers and selections.

use std::sync::Arc;

use iced::advanced::graphics::text::{Raw, Renderer as _};
use iced::advanced::image::{Handle, Image, Renderer as _};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::operation::{self, Focusable};
use iced::advanced::widget::{self, Id, Widget, tree};
use iced::advanced::{Clipboard, Shell, clipboard};
use iced::keyboard::{self, Key, key::Named};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};

use crate::layout::{Caret, Direction, Page, Reach, Stop, Target, reach};
use crate::theme;

/// How far the focus ring sits outside a link, and how thick it is.
const RING_GAP: f32 = 2.0;
const RING_WIDTH: f32 = 2.0;

pub struct Document<'a, Message> {
    id: Option<Id>,
    /// Which document the page is of: a new page of the same one (a new width, new images) keeps focus.
    document: usize,
    page: Arc<Page>,
    /// Shown images: their page-space boxes and the pixels to draw there.
    pictures: Vec<(Rectangle, Handle)>,
    theme: theme::Theme,
    on: Handlers<'a, Message>,
}

/// The messages the document sends.
pub struct Handlers<'a, Message> {
    pub follow: Box<dyn Fn(String) -> Message + 'a>,
    /// A click on an image's clickable note: the image source to load.
    pub load_image: Box<dyn Fn(String) -> Message + 'a>,
    pub hover: Box<dyn Fn(Option<String>) -> Message + 'a>,
    /// Focus moved to a stop at these heights of the page (top, bottom): bring it into view.
    pub reveal: Box<dyn Fn(f32, f32) -> Message + 'a>,
}

impl<'a, Message> Document<'a, Message> {
    pub fn new(
        document: usize,
        page: Arc<Page>,
        pictures: Vec<(Rectangle, Handle)>,
        theme: theme::Theme,
        on: Handlers<'a, Message>,
    ) -> Self {
        Document { id: None, document, page, pictures, theme, on }
    }

    /// Names the page so it can be focused by id.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Default)]
struct State {
    /// Which page the selection belongs to; a new page clears it.
    page: usize,
    document: usize,
    /// The page has the keyboard. With no `stop`, it is the page itself that is focused.
    focused: bool,
    /// The focused link or image note, by position in `stops`.
    stop: Option<usize>,
    /// The page's stops, found once per page.
    stops: Vec<Stop>,
    anchor: Option<Caret>,
    head: Option<Caret>,
    dragging: bool,
    hovered: Option<String>,
    /// Highlight rectangles for the current selection, rebuilt only when it changes.
    highlight: Vec<Rectangle>,
}

fn page_id(page: &Arc<Page>) -> usize {
    Arc::as_ptr(page) as usize
}

impl State {
    /// The state for `page` of `document`. The page keeps the keyboard if it had it, and keeps
    /// its stop if this is the same document and the page still has that stop; the selection
    /// does not carry over.
    fn on(page: &Arc<Page>, document: usize, previous: Option<&State>) -> State {
        let stops = page.stops();
        let same = previous.filter(|before| before.document == document);
        State {
            page: page_id(page),
            document,
            focused: previous.is_some_and(|before| before.focused),
            stop: same.and_then(|before| before.stop).filter(|&stop| stop < stops.len()),
            stops,
            ..State::default()
        }
    }

    /// What Enter does on the focused stop.
    fn target(&self) -> Option<&Target> {
        self.stop.and_then(|stop| self.stops.get(stop)).map(|stop| &stop.target)
    }

    /// Tab (or Shift+Tab) on the page: true when it moved to a stop, so the key is used up;
    /// false when focus should leave the page.
    fn tab(&mut self, direction: Direction) -> bool {
        match reach(self.stops.len(), self.stop, direction) {
            Reach::Stop(stop) => {
                self.stop = Some(stop);
                true
            }
            Reach::Leave => false,
        }
    }

    /// The rectangles to ring: those of the focused stop.
    fn ring(&self) -> &[Rectangle] {
        self.stop.and_then(|stop| self.stops.get(stop)).map_or(&[], |stop| &stop.rects)
    }
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.stop = None;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.stop = None;
    }
}

/// Items whose page-space bounds intersect the viewport (given in page space).
pub fn visible<'p, T>(
    items: &'p [T],
    bounds: impl Fn(&T) -> Rectangle + 'p,
    viewport: Rectangle,
) -> impl Iterator<Item = &'p T> {
    items.iter().filter(move |item| bounds(item).intersects(&viewport))
}

impl<Message> Document<'_, Message> {
    /// A left press: follow a link, act on an image note, or start a selection.
    fn pressed(&self, state: &mut State, point: Point, shell: &mut Shell<'_, Message>) {
        if let Some(href) = self.page.link_at(point) {
            shell.publish((self.on.follow)(href.to_string()));
        } else if let Some(src) = self.page.action_at(point) {
            shell.publish((self.on.load_image)(src.to_string()));
        } else {
            let caret = self.page.caret_at(point);
            self.select(state, caret, caret);
            state.dragging = true;
            shell.request_redraw();
        }
        shell.capture_event();
    }

    /// The pointer moved: extend a drag selection, or report the link under it.
    fn moved(
        &self,
        state: &mut State,
        (position, cursor): (Point, mouse::Cursor),
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        if state.dragging {
            let head = self.page.caret_at(Self::local(position, bounds));
            self.select(state, state.anchor, head);
            shell.request_redraw();
            return;
        }
        let hovered = cursor.position_in(bounds).and_then(|p| self.page.link_at(p));
        if hovered != state.hovered.as_deref() {
            state.hovered = hovered.map(str::to_string);
            shell.publish((self.on.hover)(state.hovered.clone()));
        }
    }

    /// Ctrl/Cmd+C copies the selection; Ctrl/Cmd+A selects everything.
    fn command_key(&self, state: &mut State, key: &str, clipboard: &mut dyn Clipboard, shell: &mut Shell<'_, Message>) {
        match key {
            "c" => {
                if let Some((a, b)) = self.selection(state) {
                    clipboard.write(clipboard::Kind::Standard, self.page.selected_text(a, b));
                    shell.capture_event();
                }
            }
            "a" => {
                if let Some((a, b)) = self.page.everything() {
                    self.select(state, Some(a), Some(b));
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn selection(&self, state: &State) -> Option<(Caret, Caret)> {
        state.anchor.zip(state.head).filter(|(a, b)| a != b)
    }

    fn select(&self, state: &mut State, anchor: Option<Caret>, head: Option<Caret>) {
        (state.anchor, state.head) = (anchor, head);
        state.highlight = self.selection(state).map(|(a, b)| self.page.highlight(a, b)).unwrap_or_default();
    }

    fn local(point: Point, bounds: Rectangle) -> Point {
        Point::new(point.x - bounds.x, point.y - bounds.y)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Document<'_, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.page.height))
    }

    fn layout(&mut self, _tree: &mut widget::Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, Length::Fill, self.page.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::on(&self.page, self.document, None))
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let offset = Vector::new(bounds.x, bounds.y);
        let Some(view) = viewport.intersection(&bounds) else { return };
        let in_page = view - offset;
        for fill in visible(&self.page.fills, |fill| fill.bounds, in_page) {
            renderer.fill_quad(Quad { bounds: fill.bounds + offset, ..Quad::default() }, fill.color);
        }
        if state.page == page_id(&self.page) {
            for rect in visible(&state.highlight, |rect| *rect, in_page) {
                renderer.fill_quad(Quad { bounds: *rect + offset, ..Quad::default() }, self.theme.selection);
            }
        }
        if state.focused && state.page == page_id(&self.page) {
            let border = Border { color: self.theme.ring(), width: RING_WIDTH, radius: RING_GAP.into() };
            for rect in visible(state.ring(), |rect| *rect, in_page) {
                let outer = Rectangle {
                    x: rect.x - RING_GAP,
                    y: rect.y - RING_GAP,
                    width: rect.width + 2.0 * RING_GAP,
                    height: rect.height + 2.0 * RING_GAP,
                };
                renderer.fill_quad(Quad { bounds: outer + offset, border, ..Quad::default() }, Color::TRANSPARENT);
            }
        }
        for (bounds, handle) in visible(&self.pictures, |(bounds, _)| *bounds, in_page) {
            renderer.draw_image(Image::new(handle.clone()), *bounds + offset, view);
        }
        for text in visible(&self.page.texts, |text| text.bounds, in_page) {
            renderer.fill_raw(Raw {
                buffer: Arc::downgrade(&text.buffer),
                position: text.position + offset,
                color: self.theme.ink,
                clip_bounds: view,
            });
        }
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if state.page != page_id(&self.page) {
            *state = State::on(&self.page, self.document, Some(state));
        }
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(point) = cursor.position_in(bounds) {
                    self.pressed(state, point, shell);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => state.dragging = false,
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                self.moved(state, (*position, cursor), bounds, shell);
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Tab), modifiers, .. })
                if state.focused && !modifiers.command() && !modifiers.alt() && !shell.is_event_captured() =>
            {
                let direction = if modifiers.shift() { Direction::Back } else { Direction::Forward };
                if state.tab(direction) {
                    if let Some(line) = state.ring().first() {
                        shell.publish((self.on.reveal)(line.y, line.y + line.height));
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Enter), modifiers, .. })
                if state.focused && !modifiers.command() && !modifiers.alt() && !shell.is_event_captured() =>
            {
                match state.target() {
                    Some(Target::Link(href)) => shell.publish((self.on.follow)(href.clone())),
                    Some(Target::Image(src)) => shell.publish((self.on.load_image)(src.clone())),
                    None => return,
                }
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Character(c), modifiers, .. })
                if modifiers.command() && !shell.is_event_captured() =>
            {
                self.command_key(state, c, clipboard, shell);
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        match cursor.position_in(layout.bounds()) {
            Some(p) if self.page.link_at(p).is_some() || self.page.action_at(p).is_some() => {
                mouse::Interaction::Pointer
            }
            Some(_) => mouse::Interaction::Text,
            None => mouse::Interaction::default(),
        }
    }
}

impl<'a, Message: 'a> From<Document<'a, Message>> for Element<'a, Message> {
    fn from(document: Document<'a, Message>) -> Self {
        Element::new(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::tests::page_at;
    use crate::theme::ReadingFont;

    fn page(source: &str) -> Arc<Page> {
        Arc::new(page_at(source, ReadingFont::Literata, 400.0))
    }

    const THREE_LINKS: &str = "[a](a.md) and [b](b.md)\n\n[c](c.md)";

    fn focused(page: &Arc<Page>) -> State {
        let mut state = State::on(page, 1, None);
        state.focus();
        state
    }

    #[test]
    fn tab_steps_through_the_links_then_lets_go() {
        let mut state = focused(&page(THREE_LINKS));
        assert_eq!(state.stop, None, "the page itself first");
        let seen: Vec<_> = (0..3)
            .map(|_| {
                assert!(state.tab(Direction::Forward));
                state.stop
            })
            .collect();
        assert_eq!(seen, [Some(0), Some(1), Some(2)]);
        assert!(!state.tab(Direction::Forward), "past the last link focus leaves");
    }

    #[test]
    fn shift_tab_steps_back_and_leaves_before_the_first_link() {
        let mut state = focused(&page(THREE_LINKS));
        state.stop = Some(2);
        assert!(state.tab(Direction::Back));
        assert_eq!(state.stop, Some(1));
        state.stop = Some(0);
        assert!(!state.tab(Direction::Back));
    }

    #[test]
    fn a_page_without_links_lets_tab_go_at_once() {
        let mut state = focused(&page("Just words."));
        assert!(!state.tab(Direction::Forward));
        assert!(!state.tab(Direction::Back));
    }

    #[test]
    fn the_ring_is_the_focused_links_lines_and_nothing_when_on_the_page_itself() {
        let long = page(&format!("[{}](a.md) and [b](b.md)", "word ".repeat(40)));
        let mut state = focused(&long);
        assert!(state.ring().is_empty());
        state.tab(Direction::Forward);
        assert!(state.ring().len() > 2, "every line of the wrapped link");
        state.tab(Direction::Forward);
        assert_eq!(state.ring().len(), 1);
    }

    #[test]
    fn unfocusing_drops_the_stop() {
        let mut state = focused(&page(THREE_LINKS));
        state.tab(Direction::Forward);
        state.unfocus();
        assert!((state.is_focused(), state.stop) == (false, None));
    }

    #[test]
    fn the_same_document_at_a_new_width_keeps_focus_on_the_same_link() {
        let mut before = focused(&page(THREE_LINKS));
        before.tab(Direction::Forward);
        before.tab(Direction::Forward);
        let narrower = Arc::new(page_at(THREE_LINKS, ReadingFont::Literata, 200.0));
        let after = State::on(&narrower, 1, Some(&before));
        assert_eq!((after.focused, after.stop), (true, Some(1)));
    }

    #[test]
    fn a_different_document_keeps_the_keyboard_but_not_the_stop() {
        let mut before = focused(&page(THREE_LINKS));
        before.tab(Direction::Forward);
        let after = State::on(&page(THREE_LINKS), 2, Some(&before));
        assert_eq!((after.focused, after.stop), (true, None));
    }

    #[test]
    fn a_page_that_never_had_the_keyboard_does_not_take_it() {
        let before = State::on(&page(THREE_LINKS), 1, None);
        assert!(!State::on(&page(THREE_LINKS), 2, Some(&before)).focused);
    }

    #[test]
    fn enter_acts_on_the_focused_stop_only() {
        let mut state = focused(&page("[a](a.md) ![x](x.png)"));
        assert_eq!(state.target(), None, "the page itself has nothing to follow");
        state.tab(Direction::Forward);
        assert_eq!(state.target(), Some(&Target::Link("a.md".into())));
        state.unfocus();
        assert_eq!(state.target(), None);
    }

    #[test]
    fn a_stop_the_new_page_no_longer_has_is_dropped_but_the_page_keeps_focus() {
        let mut before = focused(&page(THREE_LINKS));
        before.stop = Some(2);
        let after = State::on(&page("[only](a.md)"), 1, Some(&before));
        assert_eq!((after.focused, after.stop), (true, None));
    }

    #[test]
    fn selection_does_not_carry_over_to_a_new_page() {
        let mut before = focused(&page(THREE_LINKS));
        before.anchor = Some(Caret { fragment: 0, byte: 0 });
        let after = State::on(&page(THREE_LINKS), 1, Some(&before));
        assert_eq!(after.anchor, None);
    }

    #[test]
    fn culls_to_the_viewport() {
        let rects = [0.0, 100.0, 200.0, 300.0].map(|y| Rectangle { x: 0.0, y, width: 10.0, height: 50.0 });
        let viewport = Rectangle { x: 0.0, y: 120.0, width: 10.0, height: 100.0 };
        let shown: Vec<f32> = visible(&rects, |r| *r, viewport).map(|r| r.y).collect();
        assert_eq!(shown, [100.0, 200.0]);
    }
}
