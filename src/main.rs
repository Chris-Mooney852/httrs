use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, fmt, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

struct App {
    response: String,
    input_mode: InputMode,
    url: String,
}

impl Default for App {
    fn default() -> App {
        App {
            response: String::new(),
            input_mode: InputMode::Normal,
            url: String::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        let response = get_request(&app.url);
                        app.response = match response {
                            Ok(body) => body,
                            Err(e) => panic!("Error: {:?}", e),
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Char(c) => {
                        app.url.push(c);
                    }
                    KeyCode::Backspace => {
                        app.url.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(5), Constraint::Percentage(90)].as_ref())
        .split(f.size());

    // Top two inner blocks
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(chunks[0]);

    // Top left inner block with green background
    let input = Paragraph::new("GET").block(Block::default().borders(Borders::ALL).title("Method"));
    f.render_widget(input, top_chunks[0]);

    // Top right inner block with styled title aligned to the right
    let input = Paragraph::new(app.url.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("URL"));
    f.render_widget(input, top_chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                top_chunks[1].x + app.url.width() as u16 + 1,
                // Move one line down, from the border to the input line
                top_chunks[1].y + 1,
            )
        }
    }

    // Bottom two inner blocks
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(chunks[1]);

    // Bottom left block with all default borders
    let block = Block::default().title("Place Holder").borders(Borders::ALL);
    f.render_widget(block, bottom_chunks[0]);

    // Bottom right block with styled left and right border
    let response = Paragraph::new(app.response.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Response"));
    f.render_widget(response, bottom_chunks[1]);
}

fn get_request(url: &String) -> Result<String, Box<dyn Error>> {
    let res = reqwest::blocking::get(url)?;

    Ok(res.status().to_string())
}
