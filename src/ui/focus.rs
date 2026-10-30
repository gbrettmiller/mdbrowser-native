//! Keyboard stops for iced's own controls. A `button` or `pick_list` can't take focus, so
//! this wraps one: it becomes a stop in Tab's order, shows the ring, and turns Enter and
//! Space (a button) or Up and Down (a choice) into the message a click would send.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::{Border, Color, Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector, mouse, widget};

/// The ring sits this far clear of the control, so it shows against a button of its own colour.
const RING_GAP: f32 = 2.0;
const RING_WIDTH: f32 = 2.0;
const RING_RADIUS: f32 = 4.0;

/// A button that Tab can reach and Enter or Space presses. A disabled one (no message) is skipped.
pub fn button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    pressable(widget::button(content).on_press_maybe(on_press.clone()), on_press)
}

/// The same for a button built by hand (to style it): `button` must carry the same `on_press`.
pub fn pressable<'a, Message: Clone + 'a>(
    button: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let reachable = on_press.is_some();
    Element::new(Stop { content: button.into(), keys: Keys { press: on_press, previous: None, next: None }, reachable })
}

/// A choice that Tab can reach and Up and Down change, stopping at the ends. `previous` and
/// `next` are what picking the option before or after would send, if there is one.
pub fn choice<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    previous: Option<Message>,
    next: Option<Message>,
) -> Element<'a, Message> {
    Element::new(Stop { content: content.into(), keys: Keys { press: None, previous, next }, reachable: true })
}

/// The option before or after `current`, if there is one: where Up and Down take a choice.
pub fn neighbour<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> Option<T> {
    let at = all.iter().position(|&option| option == current)?;
    if forward { all.get(at + 1) } else { at.checked_sub(1).and_then(|before| all.get(before)) }.copied()
}

/// What a stop sends for each key.
struct Keys<Message> {
    press: Option<Message>,
    previous: Option<Message>,
    next: Option<Message>,
}

impl<Message> Keys<Message> {
    /// The message `key` asks for, if this stop answers to it. Chords with a modifier are not ours.
    fn message(&self, key: &Key, modifiers: Modifiers) -> Option<&Message> {
        if !modifiers.is_empty() {
            return None;
        }
        match key {
            Key::Named(Named::Enter | Named::Space) => self.press.as_ref(),
            Key::Named(Named::ArrowUp) => self.previous.as_ref(),
            Key::Named(Named::ArrowDown) => self.next.as_ref(),
            _ => None,
        }
    }
}

struct Stop<'a, Message> {
    content: Element<'a, Message>,
    keys: Keys<Message>,
    reachable: bool,
}

#[derive(Default)]
struct State {
    focused: bool,
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Stop<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let state = tree.state.downcast_mut::<State>();
        if self.reachable {
            operation.focusable(None, layout.bounds(), state);
        } else {
            state.unfocus();
        }
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        match event {
            // Clicking anywhere hands the keyboard back to the mouse.
            Event::Mouse(mouse::Event::ButtonPressed(_)) if state.focused => {
                state.focused = false;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) if state.focused => {
                if let Some(message) = self.keys.message(key, *modifiers) {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
        if tree.state.downcast_ref::<State>().focused {
            let bounds = layout.bounds();
            // The quad's border is drawn inside it, so the quad reaches out gap plus width.
            let reach = RING_GAP + RING_WIDTH;
            let ring = Rectangle {
                x: bounds.x - reach,
                y: bounds.y - reach,
                width: bounds.width + 2.0 * reach,
                height: bounds.height + 2.0 * reach,
            };
            let border = Border { color: theme.palette().primary, width: RING_WIDTH, radius: RING_RADIUS.into() };
            renderer.fill_quad(Quad { bounds: ring, border, ..Quad::default() }, Color::TRANSPARENT);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, Message: Clone + 'a> From<Stop<'a, Message>> for Element<'a, Message> {
    fn from(stop: Stop<'a, Message>) -> Self {
        Element::new(stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(
        press: Option<&'static str>,
        previous: Option<&'static str>,
        next: Option<&'static str>,
    ) -> Keys<&'static str> {
        Keys { press, previous, next }
    }

    fn named(key: Named) -> Key {
        Key::Named(key)
    }

    #[test]
    fn enter_and_space_press_a_button() {
        let button = keys(Some("press"), None, None);
        for key in [Named::Enter, Named::Space] {
            assert_eq!(button.message(&named(key), Modifiers::empty()), Some(&"press"));
        }
        assert_eq!(button.message(&named(Named::ArrowDown), Modifiers::empty()), None);
    }

    #[test]
    fn up_and_down_change_a_choice_and_enter_does_nothing() {
        let choice = keys(None, Some("before"), Some("after"));
        assert_eq!(choice.message(&named(Named::ArrowUp), Modifiers::empty()), Some(&"before"));
        assert_eq!(choice.message(&named(Named::ArrowDown), Modifiers::empty()), Some(&"after"));
        assert_eq!(choice.message(&named(Named::Enter), Modifiers::empty()), None);
    }

    #[test]
    fn a_choice_at_an_end_does_nothing_that_way() {
        let first = keys(None, None, Some("after"));
        assert_eq!(first.message(&named(Named::ArrowUp), Modifiers::empty()), None);
    }

    #[test]
    fn chords_with_a_modifier_are_left_for_the_shortcuts() {
        let button = keys(Some("press"), None, None);
        for modifiers in [Modifiers::CTRL, Modifiers::ALT, Modifiers::SHIFT, Modifiers::LOGO] {
            assert_eq!(button.message(&named(Named::Enter), modifiers), None, "{modifiers:?}");
        }
    }

    #[test]
    fn other_keys_do_nothing() {
        let button = keys(Some("press"), Some("before"), Some("after"));
        assert_eq!(button.message(&Key::Character("a".into()), Modifiers::empty()), None);
        assert_eq!(button.message(&named(Named::Tab), Modifiers::empty()), None);
    }

    #[test]
    fn a_choice_steps_to_its_neighbours_and_stops_at_the_ends() {
        let all = [1, 2, 3];
        assert_eq!(neighbour(&all, 2, true), Some(3));
        assert_eq!(neighbour(&all, 2, false), Some(1));
        assert_eq!(neighbour(&all, 3, true), None);
        assert_eq!(neighbour(&all, 1, false), None);
        assert_eq!(neighbour(&all, 9, true), None, "not one of the options");
    }

    #[test]
    fn a_focus_stop_takes_and_gives_up_focus() {
        let mut state = State::default();
        assert!(!state.is_focused());
        state.focus();
        assert!(state.is_focused());
        state.unfocus();
        assert!(!state.is_focused());
    }
}
