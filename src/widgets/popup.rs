///
/// Popup widget to wrap a CommandBar in a popup
///
use tui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::command_bar::{CommandBar, EventHandlerResult, InputMode};
use crate::key_hook::key_hook::KeyHook;

/// A Popup widget that wraps a CommandBar in a popup or dialog
pub struct Popup<'a> {
    /// Whether the popup should be shown
    pub show_popup: bool,
    /// command_bar is the CommandBar widget
    pub command_bar: CommandBar<'a>,
}

/// Overriding derivable_impls clippy to explictly show how the fields
/// are initialized.
#[allow(clippy::derivable_impls)]
impl<'a> Default for Popup<'a> {
    fn default() -> Popup<'a> {
        Popup {
            command_bar: CommandBar::default(),
            show_popup: false,
        }
    }
}

impl<'a> KeyHook<'a, CommandBar<'a>> for Popup<'a> {
    fn register_key(&mut self, key: char, f: &'a dyn Fn(&mut CommandBar<'a>, char)) {
        self.command_bar.command_key = Some(key);
        self.command_bar.key_database.keys.insert(key, f);
    }

    fn unregister_key(&mut self, key: char) {
        self.command_bar.key_database.keys.remove(&key);

        // Unset the command key if it matches
        if let Some(command_key) = self.command_bar.command_key {
            if command_key == key {
                self.command_bar.command_key = None;
            }
        }
    }
}

impl<'a> Popup<'a> {
    /// Handle an event
    /// If the widget is not registered to handle the event, pass it to the parent
    pub fn handle_event(&mut self) -> EventHandlerResult {
        let res = self.command_bar.handle_event();
        if res == EventHandlerResult::Ok {
            match self.command_bar.input_mode {
                InputMode::Normal => {
                    self.show_popup = false;
                }
                InputMode::Editing => {
                    self.show_popup = true;
                }
            }
        }

        res
    }
}

impl<'a> Widget for Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.command_bar.render(area, buf);
    }
}

impl<'a> Widget for &mut Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let command_bar = &mut self.command_bar;
        command_bar.render(area, buf);
    }
}

impl<'a> Widget for &'a Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let command_bar = &self.command_bar;
        command_bar.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::command_bar::CommandBar;
    use crate::widgets::popup::Popup;

    #[test]
    fn popup_default() {
        let popup = Popup::default();
        let command_bar = CommandBar::default();

        assert!(!popup.show_popup);
        assert_eq!(popup.command_bar.command_key, command_bar.command_key);
        assert_eq!(popup.command_bar.input, command_bar.input);
        assert_eq!(popup.command_bar.input_mode, command_bar.input_mode);
        assert_eq!(popup.command_bar.messages, command_bar.messages);
        assert_eq!(popup.command_bar.width, command_bar.width);
    }
}
