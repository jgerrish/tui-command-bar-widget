#[cfg(test)]
use tui::{backend::TestBackend, buffer::Buffer, layout::Rect, style::Color, Terminal};

use mockall::*;

use tui_command_bar_widget::widgets::command_bar::CommandBar;

use std::sync::Mutex;

lazy_static! {
    /// The context object is vulnerable to race conditions
    /// Use a mutex so only one test at a time has access to update it
    /// The lock will last for the length of the let block in the tests
    /// Based on the mock_struct_with_static_method.rs tests
    static ref EVENT_READ_MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn command_bar_renders() {
    let backend = TestBackend::new(40, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let area = Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 3,
            };
            let mut command_bar_widget = CommandBar::default();
            frame.render_widget(&mut command_bar_widget, area);
        })
        .unwrap();
    let mut expected = Buffer::with_lines(vec![
        "┌Command───────────────────────────────┐",
        "│                                      │",
        "└──────────────────────────────────────┘",
    ]);
    for x in 1..=7 {
        expected.get_mut(x, 0).set_fg(Color::Reset);
    }
    terminal.backend().assert_buffer(&expected);
}
