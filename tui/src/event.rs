use std::io;
use std::time::Duration;

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent, KeyEvent};

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub fn read() -> io::Result<Event> {
    if !crossterm_event::poll(POLL_TIMEOUT)? {
        return Ok(Event::Tick);
    }

    match crossterm_event::read()? {
        CrosstermEvent::Key(key) => Ok(Event::Key(key)),
        CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
        _ => Ok(Event::Tick),
    }
}
