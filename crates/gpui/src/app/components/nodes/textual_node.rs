use std::f32::INFINITY;

use gpui::{App, Context, Entity, SharedString, Window};
use gpui_component::input::{InputState, Position};
use uuid::Uuid;

use crate::app::{components::slash_menu::SlashMenu, states::node_state::NodeState};

/// Events emitted by a TextualNode during user interaction.
#[derive(Debug, Clone)]
pub enum TextualNodeEvent {
    /// The content has changed. Contains the new content.
    Change(SharedString),
    /// The content became empty (was non-empty before, now empty).
    Empty,
    /// The "/" character was typed (triggers slash menu).
    SlashTyped,
    /// The Backspace key was pressed.
    Backspace,
    /// The Delete key was pressed.
    Delete,
    /// The Enter key was pressed.
    Enter,
    /// The node received focus.
    Focus,
    /// The node lost focus.
    Blur,
}

/// Trait for nodes that contain an editable text field.
///
/// This trait provides the basic interface for text-based nodes like TextNode and HeadingNode.
/// It defines accessors for the input state, node state, and content management.
pub trait TextualNode {
    /// Returns a reference to the input state entity.
    fn input_state(&self) -> &Entity<InputState>;

    /// Returns a reference to the node state entity.
    fn node_state(&self) -> &Entity<NodeState>;

    /// Returns the unique identifier of this node.
    fn node_id(&self) -> Uuid;

    /// Returns the current text content of the node.
    fn content(&self) -> SharedString;

    /// Sets the text content of the node.
    fn set_content(&mut self, content: SharedString);

    /// Returns whether the node is currently focused.
    fn is_focused(&self) -> bool;

    /// Sets the focused state of the node.
    fn set_focused(&mut self, focused: bool);

    /// Focuses the input field.
    fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input_state().update(cx, |element, cx| {
            element.focus(window, cx);
        });
    }

    /// Moves the cursor to the end of the content.
    fn move_cursor_end(&self, window: &mut Window, cx: &mut App) {
        self.input_state().update(cx, |element, cx| {
            element.set_cursor_position(
                Position::new(INFINITY as u32, INFINITY as u32),
                window,
                cx,
            );
        });
    }
}

/// Trait for nodes that support the slash menu (triggered by typing "/").
///
/// This trait extends TextualNode to add slash menu functionality.
pub trait SlashMenuNode: TextualNode {
    /// Returns a reference to the slash menu entity.
    fn slash_menu(&self) -> &Entity<SlashMenu>;

    /// Returns whether the slash menu is currently open.
    fn is_menu_open(&self, cx: &App) -> bool {
        self.slash_menu().read(cx).open
    }

    /// Opens or closes the slash menu.
    fn set_menu_open(&self, open: bool, window: &mut Window, cx: &mut App) {
        self.slash_menu().update(cx, |menu, cx| {
            menu.set_open(open, window, cx);
        });
    }
}

/// Delegate trait for reacting to textual node events.
///
/// Implement this trait to handle events emitted by a TextualNode.
/// This allows each node type to define its own behavior in response to events.
pub trait TextualNodeDelegate: TextualNode + Sized {
    /// Called when a textual event occurs.
    ///
    /// Implement this method to handle various events like content changes,
    /// key presses (Enter, Backspace, Delete), slash menu triggers, etc.
    fn on_textual_event(
        &mut self,
        event: TextualNodeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    );

    /// Handles focus event: updates the focused state and emits the Focus event.
    fn handle_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_focused(true);
        self.on_textual_event(TextualNodeEvent::Focus, window, cx);
    }

    /// Handles blur event: updates the focused state and emits the Blur event.
    fn handle_blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_focused(false);
        self.on_textual_event(TextualNodeEvent::Blur, window, cx);
    }
}
