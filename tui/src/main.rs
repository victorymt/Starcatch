use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod app;
mod components;
mod event;
mod handler;
mod styles;
mod ui;

fn main() -> io::Result<()> {
    // Parse args for optional db path
    let db_path = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-'))
        .unwrap_or_else(default_db_path);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = match app::App::new(&db_path) {
        Ok(app) => app,
        Err(err) => {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Error initializing app: {}", err);
            std::process::exit(1);
        }
    };

    // Run event loop
    let res = run(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut app::App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let evt = event::read()?;
        if handler::handle(app, evt)? {
            break; // quit
        }

        // Auto-refresh data on every tick if something changed
        if app.needs_refresh {
            app.refresh_data();
            app.needs_refresh = false;
        }
    }
    Ok(())
}

fn default_db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!(
        "{}/.local/share/starcatch/starcatch.db",
        home
    )
}
