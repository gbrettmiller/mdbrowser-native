//! The document widget: paints a laid-out `Page` (shaped once, at layout) and turns
//! pointer and keyboard input into link follows, hovers and selections.

use std::sync::Arc;

use iced::advanced::graphics::text::{Raw, Renderer as _};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{self, Widget, tree};
use iced::advanced::{Clipboard, Shell, clipboard};
use iced::keyboard::{self, Key};
use iced::{Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};

use crate::layout::{Caret, Page};
use crate::theme;

pub struct Document<'a, Message> {
    page: Arc<Page>,
    theme: theme::Theme,
    on_follow: Box<dyn Fn(String) -> Message + 'a>,
    on_hover: Box<dyn Fn(Option<String>) -> Message + 'a>,
}

impl<'a, Message> Document<'a, Message> {
    pub fn new(
        page: Arc<Page>,
        theme: theme::Theme,
        on_follow: impl Fn(String) -> Message + 'a,
        on_hover: impl Fn(Option<String>) -> Message + 'a,
    ) -> Self {
        Document { page, theme, on_follow: Box::new(on_follow), on_hover: Box::new(on_hover) }
    }
}

#[derive(Default)]
struct State {
    /// Which page the selection belongs to; a new page clears it.
    page: usize,
    anchor: Option<Caret>,
    head: Option<Caret>,
    dragging: bool,
    hovered: Option<String>,
}

fn page_id(page: &Arc<Page>) -> usize {
    Arc::as_ptr(page) as usize
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
    fn selection(&self, state: &State) -> Option<(Caret, Caret)> {
        state.anchor.zip(state.head).filter(|(a, b)| a != b)
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
        tree::State::new(State { page: page_id(&self.page), ..State::default() })
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
        if let Some((a, b)) = self.selection(state).filter(|_| state.page == page_id(&self.page)) {
            for rect in self.page.highlight(a, b).into_iter().filter(|rect| rect.intersects(&in_page)) {
                renderer.fill_quad(Quad { bounds: rect + offset, ..Quad::default() }, self.theme.selection);
            }
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
            *state = State { page: page_id(&self.page), ..State::default() };
        }
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(point) = cursor.position_in(bounds) else { return };
                if let Some(href) = self.page.link_at(point) {
                    shell.publish((self.on_follow)(href.to_string()));
                } else {
                    state.anchor = self.page.caret_at(point);
                    state.head = state.anchor;
                    state.dragging = true;
                    shell.request_redraw();
                }
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => state.dragging = false,
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.dragging {
                    state.head = self.page.caret_at(Self::local(*position, bounds));
                    shell.request_redraw();
                    return;
                }
                let hovered = cursor.position_in(bounds).and_then(|p| self.page.link_at(p)).map(str::to_string);
                if hovered != state.hovered {
                    state.hovered = hovered.clone();
                    shell.publish((self.on_hover)(hovered));
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Character(c), modifiers, .. })
                if modifiers.command() && !shell.is_event_captured() =>
            {
                match c.as_str() {
                    "c" => {
                        if let Some((a, b)) = self.selection(state) {
                            clipboard.write(clipboard::Kind::Standard, self.page.selected_text(a, b));
                            shell.capture_event();
                        }
                    }
                    "a" => {
                        if let Some((a, b)) = self.page.everything() {
                            (state.anchor, state.head) = (Some(a), Some(b));
                            shell.request_redraw();
                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
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
            Some(p) if self.page.link_at(p).is_some() => mouse::Interaction::Pointer,
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

    #[test]
    fn culls_to_the_viewport() {
        let rects = [0.0, 100.0, 200.0, 300.0].map(|y| Rectangle { x: 0.0, y, width: 10.0, height: 50.0 });
        let viewport = Rectangle { x: 0.0, y: 120.0, width: 10.0, height: 100.0 };
        let shown: Vec<f32> = visible(&rects, |r| *r, viewport).map(|r| r.y).collect();
        assert_eq!(shown, [100.0, 200.0]);
    }
}
