/// A command bar widget
/// This is a command bar widget that lets you edit commands in a line
/// The command bar can be used in other widgets or views, such as a horizontal
/// layout or popup.
///
/// The CommandBar widget lets you register to receive commands on a channel
/// when you build the object.
use log::{debug, error};

use std::sync::{mpsc, mpsc::SendError};

// This adds a width() method to String
use ::crossterm::event::{Event, KeyCode};
use unicode_width::UnicodeWidthStr;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::key_hook::key_hook::{KeyDatabase, KeyHook};

use mockall_double::double;

#[double]
pub use crate::crossterm::event;
//pub use crate::crossterm::event;

//use crate::event;

/// A CommandBar has an InputMode that indicates it's editing state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    /// Normal means the CommandBar is not being edited
    /// Depending on the widget type, it may not be visible or it may be unfocused
    Normal,
    /// Editing means the CommandBar is in edit mode
    Editing,
}

/// CommandBar is a widget for easy editing of commands in a line.
///
/// # Example
///
/// ```
/// use ratatui::{Frame, backend::TestBackend, layout::{Layout, Rect}, Terminal};
/// use tui_command_bar_widget::widgets::command_bar::{EventHandlerResult, InputMode, CommandBar};
/// use tui_command_bar_widget::key_hook::key_hook::KeyHook;
///
/// let backend = TestBackend::new(5, 5);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let area = Rect::new(0, 0, 5, 5);
/// let mut frame = terminal.get_frame();
/// let chunks = Layout::default();
///
/// let mut command_bar_widget = CommandBar::default();
/// let mut closure = |cb: &mut CommandBar, key| { cb.command_key_handler(key) };
/// command_bar_widget.register_key(':', &mut closure);
/// frame.render_widget(command_bar_widget, area);
///
/// ```
#[derive(Clone)]
pub struct CommandBar<'a> {
    /// Command key to activate the CommandBar
    pub command_key: Option<char>,
    /// Current value of the input box
    pub input: String,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<String>,
    /// channel to use for sending messages
    pub tx_channel: Option<mpsc::Sender<String>>,
    /// Keep track of width to limit the text in the command bar
    pub width: u16,
    /// The key database to store key actions
    pub key_database: KeyDatabase<'a, CommandBar<'a>>,
}

impl<'a> Default for CommandBar<'a> {
    fn default() -> CommandBar<'a> {
        CommandBar {
            command_key: None,
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            tx_channel: None,
            width: 0,
            key_database: KeyDatabase::default(),
        }
    }
}

/// The CommandBar event handler handles UI events and returns a result
/// depending on how the event was processed
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EventHandlerResult {
    /// A result of Ok indicates the event was processed by the CommandBar
    Ok,
    /// A result of Err indicates there was an error processing the event
    /// For example, the event read call may have failed, or the event was
    /// invalid.
    Err,
    /// An Unhandled event is what that the CommandBar didn't know how to process
    Unhandled(Event),
}

impl<'a> KeyHook<'a, CommandBar<'a>> for CommandBar<'a> {
    fn register_key(&mut self, key: char, f: &'a dyn Fn(&mut Self, char)) {
        self.command_key = Some(key);
        self.key_database.keys.insert(key, f);
    }

    fn unregister_key(&mut self, key: char) {
        self.key_database.keys.remove(&key);

        // Unset the command key if it matches
        if let Some(command_key) = self.command_key {
            if command_key == key {
                self.command_key = None;
            }
        }
    }
}

impl<'a> CommandBar<'a> {
    /// Build a default CommandBar with a send channel
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::mpsc;
    /// use tui_command_bar_widget::key_hook::key_hook::KeyHook;
    /// use tui_command_bar_widget::widgets::command_bar::CommandBar;
    ///
    /// let (tx, rx) = mpsc::channel();
    /// let mut command_bar_widget = CommandBar::default_with_tx_channel(tx);
    ///
    /// // Normally this would be done with events generated on the terminal
    /// // See the unit tests for an example event stream
    /// command_bar_widget.input = String::from("some input");
    /// command_bar_widget.submit();
    /// let received = rx.recv().unwrap();
    /// assert_eq!(received, "some input");
    /// ```
    pub fn default_with_tx_channel(tx_channel: mpsc::Sender<String>) -> Self {
        CommandBar {
            tx_channel: Some(tx_channel),
            ..Default::default()
        }
    }

    /// Commit changes in the command bar and close the command bar
    pub fn submit(&mut self) -> Result<(), SendError<String>> {
        let msg: String = self.input.drain(..).collect();
        self.messages.push(msg.clone());
        match &self.tx_channel {
            Some(tx) => tx.send(msg),
            None => Ok(()),
        }
    }

    /// Change the input mode to Normal,
    /// Different widgets may hide the CommandBar or unfocus it.
    pub fn normal(&mut self) {
        debug!("Exiting editing mode");
        self.input_mode = InputMode::Normal;
    }

    /// Handle the special command key
    pub fn command_key_handler(&mut self, key: char) {
        debug!("Command key pressed: {:?}", key);
        if let InputMode::Normal = self.input_mode {
            self.input_mode = InputMode::Editing;
        }
    }

    /// Handle an event
    /// If the widget is not registered to handle the event, pass it to the parent
    pub fn handle_event(&mut self) -> EventHandlerResult {
        #[allow(unused_assignments)]
        let mut handled = false;

        let res = event::read();
        let event = match res {
            Ok(e) => e,
            Err(e) => {
                error!("Event read error: {}", e);
                return EventHandlerResult::Err;
            }
        };
        match event {
            Event::Key(key) => {
                // TODO: Match against KeyDatabase
                //       Maybe only match against KeyDatabase
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char(k) => {
                            // TODO: This needs to be refactored, there are a lot
                            // of issues around clean API design here that need to
                            // be better thought out
                            if self.key_database.keys.contains_key(&k) {
                                let value_option = self.key_database.keys.get(&k);
                                if let Some(f) = value_option {
                                    (*f)(self, k);
                                }
                            }
                            match self.command_key {
                                // No command key is registered, do nothing
                                None => {
                                    handled = false;
                                }
                                // A command key is registered, see if it matches
                                Some(c) => {
                                    if c == k {
                                        //self.input_mode = InputMode::Editing;
                                        handled = true;
                                    } else {
                                        handled = false;
                                    }
                                }
                            }
                        }
                        _ => {
                            handled = false;
                        }
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            // Entering leaves edit mode and commits the text
                            match self.submit() {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("Send error on message: {}", e);
                                }
                            }
                            self.normal();
                            handled = true;
                        }
                        KeyCode::Char(c) => {
                            if self.input.width() < self.width.into() {
                                self.input.push(c);
                            } else {
                                debug!(
                                    "Didn't input data, input too small: {}, {}",
                                    self.input.width(),
                                    self.width
                                );
                            }
                            handled = true;
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                            handled = true;
                        }
                        KeyCode::Esc => {
                            self.normal();
                            handled = true;
                        }
                        _ => {
                            handled = false;
                        }
                    },
                }
            }
            Event::Resize(w, h) => {
                debug!("Resize event: {:?}, {:?}", w, h);
                handled = false;
            }
            Event::Mouse(e) => {
                debug!("Mouse event: {:?}", e);
                handled = false;
            }
        };
        if handled {
            EventHandlerResult::Ok
        } else {
            EventHandlerResult::Unhandled(event)
        }
    }
}

impl<'a> Widget for CommandBar<'a> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}

impl<'a> Widget for &'a CommandBar<'a> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}

impl<'a> Widget for &mut CommandBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        debug!("area width: {:?}, height: {:?}", area.width, area.height);
        debug!(
            "buf width: {:?}, height: {:?}",
            buf.area.width, buf.area.height
        );

        // The render code in Paragraph selects the buffer cells based on the
        // area rectangle.
        // We set this to limit what the user can type in instead of relying on
        // the LineTruncator code.
        // Future versions could maybe scroll the text left
        self.width = area.width - 2;

        let input = Paragraph::new(self.input.clone())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Command"));

        input.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use log::debug;

    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
        Terminal,
    };

    use ::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ::crossterm::event::{MouseEvent, MouseEventKind};

    use crate::key_hook::key_hook::KeyHook;
    use crate::widgets::command_bar::{CommandBar, EventHandlerResult, InputMode};

    use std::sync::Mutex;

    use mockall::*;
    use mockall_double::double;

    use std::sync::mpsc;

    #[double]
    pub use crate::crossterm::event;

    lazy_static! {
        /// The context object is vulnerable to race conditions
        /// Use a mutex so only one test at a time has access to update it
        /// The lock will last for the length of the let block in the tests
        /// Based on the mock_struct_with_static_method.rs tests
        static ref EVENT_READ_MUTEX: Mutex<()> = Mutex::new(());
    }

    fn handle_generic_event(
        command_bar_widget: &mut CommandBar,
        event: Event,
    ) -> EventHandlerResult {
        let _m = EVENT_READ_MUTEX.lock().unwrap();

        let context = event::read_context();

        context.expect().with().returning(move || {
            return ::crossterm::Result::Ok(event);
        });

        command_bar_widget.handle_event()
    }

    fn handle_error_event(command_bar_widget: &mut CommandBar) -> EventHandlerResult {
        let _m = EVENT_READ_MUTEX.lock().unwrap();

        let context = event::read_context();

        context.expect().with().returning(move || {
            return ::crossterm::Result::Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from("read error"),
            ));
        });

        command_bar_widget.handle_event()
    }

    /// Run an event test with various parameters
    /// register_key is a key to register as a command key, or None if no key
    /// should be registered.
    /// start_mode is the editing mode to run the test in
    /// input_event is the event to queue for the event handler
    /// expected_event_result_option is the expected result from the event handler
    /// expected_input_mode_option is the expected input mode state after the
    /// event is processed.
    /// other_tests is a closure of any other tests to run against the CommandBar
    /// object.
    fn run_event_test<'a>(
        register_key: Option<char>,
        start_mode: Option<InputMode>,
        input_event: Option<Event>,
        expected_event_result_option: Option<EventHandlerResult>,
        expected_input_mode_option: Option<InputMode>,
        other_tests: Option<&'a dyn Fn(CommandBar)>,
    ) {
        let mut command_bar_widget = CommandBar::default();
        // create the closure here so it lives for as long as the CommandBar
        // TODO: Maybe we could annotate this so it's not needed
        let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);

        if let Some(k) = register_key {
            command_bar_widget.register_key(k, &mut closure);
        }

        if let Some(start_mode) = start_mode {
            command_bar_widget.input_mode = start_mode;
        }

        // If there is an input event, handle it and run any tests
        if let Some(event) = input_event {
            let event_result = handle_generic_event(&mut command_bar_widget, event);

            if let Some(expected_event_result) = expected_event_result_option {
                assert_eq!(event_result, expected_event_result);
            }

            if let Some(expected_input_mode) = expected_input_mode_option {
                assert_eq!(command_bar_widget.input_mode, expected_input_mode);
            }
        }

        // Run extra tests that were passed in
        if let Some(f) = other_tests {
            f(command_bar_widget);
        }
    }

    #[test]
    fn command_bar_registers_command_key() {
        run_event_test(
            Some(':'),
            None,
            None,
            None,
            None,
            Some(&|command_bar_widget: CommandBar| {
                assert!(command_bar_widget.command_key.is_some());
                assert_eq!(command_bar_widget.command_key.unwrap(), ':');
            }),
        );
    }

    #[test]
    fn command_bar_handles_command_key() {
        run_event_test(
            Some(':'),
            None,
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char(':'),
                KeyModifiers::NONE,
            ))),
            Some(EventHandlerResult::Ok),
            Some(InputMode::Editing),
            None,
        );
    }

    #[test]
    fn command_bar_handles_escape_key() {
        run_event_test(
            Some(':'),
            Some(InputMode::Editing),
            Some(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
            Some(EventHandlerResult::Ok),
            Some(InputMode::Normal),
            None,
        );
    }

    #[test]
    fn command_bar_handles_non_command_key_in_normal_mode() {
        run_event_test(
            Some(':'),
            None,
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            ))),
            Some(EventHandlerResult::Unhandled(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            )))),
            Some(InputMode::Normal),
            None,
        );
    }

    #[test]
    fn command_bar_unregistered_handles_command_key_in_normal_mode() {
        run_event_test(
            None,
            None,
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char(':'),
                KeyModifiers::NONE,
            ))),
            Some(EventHandlerResult::Unhandled(Event::Key(KeyEvent::new(
                KeyCode::Char(':'),
                KeyModifiers::NONE,
            )))),
            Some(InputMode::Normal),
            None,
        );
    }

    #[test]
    fn command_bar_handles_non_key_event_in_normal_mode() {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        run_event_test(
            Some(':'),
            None,
            Some(Event::Mouse(mouse_event)),
            Some(EventHandlerResult::Unhandled(Event::Mouse(mouse_event))),
            Some(InputMode::Normal),
            None,
        );
    }

    #[test]
    fn command_bar_handles_non_key_event_in_normal_mode_with_unregistered_command_key() {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        run_event_test(
            None,
            None,
            Some(Event::Mouse(mouse_event)),
            Some(EventHandlerResult::Unhandled(Event::Mouse(mouse_event))),
            Some(InputMode::Normal),
            None,
        );
    }

    #[test]
    fn command_bar_unregistered_handles_command_key_in_editing_mode() {
        // command editing via keyboard events requires a widget with valid
        // length > 0, so we need to simulate the terminal
        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect {
                    x: 0,
                    y: 0,
                    width: 40,
                    height: 4,
                };
                let mut command_bar_widget = CommandBar::default();

                frame.render_widget(&mut command_bar_widget, area);
            })
            .unwrap();
    }

    /// Handle event read errors in normal mode, with unregistered command key
    /// The current crossterm code doesn't seem to return read errors, but
    /// we'll still test against the API
    #[test]
    fn command_bar_handles_event_read_error_in_normal_mode_with_unregistered_command_key() {
        let mut command_bar_widget = CommandBar::default();

        // test that event.read returning an error is handled correctly
        let event_res = handle_error_event(&mut command_bar_widget);

        assert!(match event_res {
            EventHandlerResult::Err => true,
            _ => false,
        });
        assert!(match command_bar_widget.input_mode {
            InputMode::Normal => true,
            InputMode::Editing => false,
        });
    }

    /// Handle event read errors in normal mode
    /// The current crossterm code doesn't seem to return read errors, but
    /// we'll still test against the API
    #[test]
    fn command_bar_handles_event_read_error_in_normal_mode() {
        let mut command_bar_widget = CommandBar::default();
        let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
        command_bar_widget.register_key(':', &mut closure);

        // test that event.read returning an error is handled correctly
        let event_res = handle_error_event(&mut command_bar_widget);

        assert!(match event_res {
            EventHandlerResult::Err => true,
            _ => false,
        });
        assert!(match command_bar_widget.input_mode {
            InputMode::Normal => true,
            InputMode::Editing => false,
        });
    }

    /// Handle event read errors in editing mode
    /// The current crossterm code doesn't seem to return read errors, but
    /// we'll still test against the API
    #[test]
    fn command_bar_handles_event_read_error_in_editing_mode() {
        let mut command_bar_widget = CommandBar::default();
        let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
        command_bar_widget.register_key(':', &mut closure);

        // enter editing mode
        debug!("Entering editing mode");
        command_bar_widget.input_mode = InputMode::Editing;

        // test that event.read returning an error is handled correctly
        let event_res = handle_error_event(&mut command_bar_widget);

        assert!(match event_res {
            EventHandlerResult::Err => true,
            _ => false,
        });
        assert!(match command_bar_widget.input_mode {
            InputMode::Normal => false,
            InputMode::Editing => true,
        });
    }

    #[test]
    fn command_bar_sends_message() {
        let (tx, rx) = mpsc::channel();
        let mut command_bar_widget = CommandBar::default_with_tx_channel(tx);

        let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
        command_bar_widget.register_key(':', &mut closure);

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect {
                    x: 0,
                    y: 0,
                    width: 40,
                    height: 4,
                };
                frame.render_widget(&mut command_bar_widget, area);
            })
            .unwrap();
        let event = Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
        handle_generic_event(&mut command_bar_widget, event);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        handle_generic_event(&mut command_bar_widget, event);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        handle_generic_event(&mut command_bar_widget, event);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        handle_generic_event(&mut command_bar_widget, event);

        let received = rx.recv().unwrap();

        assert_eq!(received, "ab");
    }

    #[test]
    fn command_bar_edit_renders() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut command_bar_widget = CommandBar::default();
        let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
        command_bar_widget.register_key(':', &mut closure);

        terminal
            .draw(|frame| {
                let area = Rect {
                    x: 0,
                    y: 0,
                    width: 40,
                    height: 3,
                };
                frame.render_widget(&mut command_bar_widget, area);

                let event = Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
                handle_generic_event(&mut command_bar_widget, event);

                let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
                handle_generic_event(&mut command_bar_widget, event);

                let event = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
                handle_generic_event(&mut command_bar_widget, event);

                frame.render_widget(&mut command_bar_widget, area);
            })
            .unwrap();

        let mut expected = Buffer::with_lines(vec![
            "┌Command───────────────────────────────┐",
            "│ab                                    │",
            "└──────────────────────────────────────┘",
        ]);
        expected.set_style(Rect::new(0, 0, 40, 3), Style::default().fg(Color::Yellow));
        for x in 1..=7 {
            expected.get_mut(x, 0).set_fg(Color::Yellow);
        }
        terminal.backend().assert_buffer(&expected);
    }
}
