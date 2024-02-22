/// A simple example demonstrating how to display a command bar widget in a
/// popup.
/// This example is based on the popup example in the TUI crate.
///
/// Pressing the command key pops up the command bar in a popup box.
/// When visible, typing enters data in the command bar.
/// Pressing the escape key closes the popup box.
use std::{error::Error, io};

use config::Config;
#[allow(clippy::single_component_path_imports)]
use env_logger;
use log::{debug, error, info};

// This adds a width() method to String
use unicode_width::UnicodeWidthStr;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui_command_bar_widget::widgets::popup::Popup;

use tui_command_bar_widget::key_hook::key_hook::KeyHook;
use tui_command_bar_widget::widgets::command_bar::{CommandBar, EventHandlerResult};

fn main() -> Result<(), Box<dyn Error>> {
    // Load config
    let mut debug = true;
    let mut command_key = ':';

    // Initialize logger
    if let Err(e) = env_logger::try_init() {
        panic!("couldn't initialize logger: {:?}", e);
    }

    let settings_result = load_settings("config/tui-command-bar-widget.toml");
    match settings_result {
        Ok(settings) => {
            info!("merged in config");
            if let Ok(b) = settings.get_bool("debug") {
                debug = b;
            }
            if let Ok(k) = settings.get_string("command-key") {
                command_key = match k.chars().next() {
                    Some(c) => c,
                    None => command_key,
                };
                debug!("command_key: {}", command_key);
            }
        }
        Err(s) => {
            error!("error loading config: {:?}", s)
        }
    };

    match debug {
        true => {
            debug!("debug mode is enabled");
        }
        false => {
            debug!("debug mode is disabled");
        }
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut command_bar_widget = Popup::default();
    let closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
    command_bar_widget.register_key(command_key, &closure);
    let res = run_app(&mut terminal, command_bar_widget);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut command_bar_widget: Popup,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut command_bar_widget))?;

        match command_bar_widget.handle_event() {
            EventHandlerResult::Err => {
                return Ok(());
            }
            EventHandlerResult::Ok => {}
            EventHandlerResult::Unhandled(event) => {
                if let Event::Key(key) = event {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(());
                    } else {
                        {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, command_bar_widget: &mut Popup) {
    let size = f.size();

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(size);

    let command_key = command_bar_widget.command_bar.command_key.unwrap_or('p');

    let escape_key = "Esc";

    let text = if command_bar_widget.show_popup {
        format!("Press {} to close the popup", escape_key)
    } else {
        format!("Press {} to show the popup", command_key)
    };
    let paragraph = Paragraph::new(Span::styled(
        text,
        Style::default().add_modifier(Modifier::SLOW_BLINK),
    ))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[0]);

    let block = Block::default()
        .title("Content")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Blue));
    f.render_widget(block, chunks[1]);

    if command_bar_widget.show_popup {
        let area = fixed_height_centered_rect(80, 3, size);
        let width = command_bar_widget.command_bar.input.width();

        f.render_widget(Clear, area); // this clears out the background
        f.render_widget(command_bar_widget, area);

        f.set_cursor(area.x + width as u16 + 1, area.y + 1);
    }
}

/// helper function to create a centered rect using up certain
/// percentage of the available rect `r`
/// The vertical height is fixed.
fn fixed_height_centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// load settings from a config file
/// returns the config settings as a Config on success, or a ConfigError on failure
fn load_settings(config_name: &str) -> Result<Config, config::ConfigError> {
    Config::builder()
        // Add in config file
        .add_source(config::File::with_name(config_name))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/command_bar_widget would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
}
