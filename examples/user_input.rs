/// A simple example demonstrating how to display a command bar widget in
/// a vertical layout.
/// This example is based on the user_input example in the TUI crate.
///
/// Pressing the command key focuses the command bar.
/// When focused, typing enters data.
/// Pressing the escape key removes focus.
use std::{error::Error, io};

use config::Config;
use env_logger;
use log::{debug, error, info};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use tui_command_bar_widget::key_hook::key_hook::KeyHook;
use tui_command_bar_widget::widgets::command_bar::{CommandBar, EventHandlerResult, InputMode};

pub struct App {
    /// History of recorded messages
    pub messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            messages: Vec::new(),
        }
    }
}

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

    let app = App::default();

    // create command_bar_widget and run it
    let mut command_bar_widget = CommandBar::default();
    let mut closure = |cb: &mut CommandBar, key| cb.command_key_handler(key);
    command_bar_widget.register_key(command_key, &mut closure);

    let res = run_app(&mut terminal, app, command_bar_widget);

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
    mut app: App,
    mut command_bar_widget: CommandBar,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app, &mut command_bar_widget))?;

        // TODO: refactor into proper event handling tree
        match command_bar_widget.handle_event() {
            // The widget returned an error, quit the event loop
            EventHandlerResult::Err => {
                return Ok(());
            }
            // The widget handled the event, continue processing events
            EventHandlerResult::Ok => {}
            // The widget didn't know how to handle the event, so we should
            EventHandlerResult::Unhandled(event) => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    };
                };
            }
        }
    }
}

/// UI event loop function
/// This may be run on every iteration of the event loop
fn ui<B: Backend>(f: &mut Frame<B>, _app: &App, command_bar_widget: &mut CommandBar) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let cmd_key = command_bar_widget.command_key.unwrap_or(':');
    let cmd_key_str = format!("{}", cmd_key);

    let (msg, style) = match command_bar_widget.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled(cmd_key_str, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);
    let width = command_bar_widget.input.len();

    if let InputMode::Editing = command_bar_widget.input_mode {
        f.set_cursor(chunks[1].x + width as u16 + 1, chunks[1].y + 1);
    }

    let messages: Vec<ListItem> = command_bar_widget
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));

    f.render_widget(command_bar_widget, chunks[1]);
    f.render_widget(messages, chunks[2]);
}

/// load settings from a config file
/// returns the config settings as a Config on success, or a ConfigError on failure
fn load_settings<'a>(config_name: &str) -> Result<Config, config::ConfigError> {
    Config::builder()
        // Add in config file
        .add_source(config::File::with_name(config_name))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/command_bar_widget would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
}
