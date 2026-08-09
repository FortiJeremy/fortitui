//! Terminal event loop.
//!
//! A dedicated thread polls crossterm for keyboard/resize events and emits a
//! `Tick` at a fixed interval. The async main loop `select!`s on this receiver
//! so the UI never blocks on terminal input nor on the refresh cadence.
//!
//! When the receiver is dropped (loop exits, e.g. on `q`), the sender errors
//! and the thread terminates itself.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

/// A UI event delivered to the main loop.
#[derive(Debug, Clone)]
pub enum Event {
    /// A key press.
    Key(KeyEvent),
    /// Terminal resized.
    Resize(u16, u16),
    /// Periodic timer tick; drives background refresh + redraw.
    Tick,
}

/// Source of UI events; call [`EventLoop::next`] in the main async loop.
pub struct EventLoop {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventLoop {
    /// Spawn the event thread. `tick_ms` is the refresh cadence.
    pub fn new(tick_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let poll = Duration::from_millis(tick_ms);
        std::thread::spawn(move || loop {
            if event::poll(poll).unwrap_or(true) {
                if let Ok(ev) = event::read() {
                    let delivered = match ev {
                        CrosstermEvent::Key(k) => tx.send(Event::Key(k)),
                        CrosstermEvent::Resize(w, h) => tx.send(Event::Resize(w, h)),
                        _ => Ok(()),
                    };
                    if delivered.is_err() {
                        break;
                    }
                }
            }
            if tx.send(Event::Tick).is_err() {
                break;
            }
        });
        Self { rx }
    }

    /// Wait for the next event (or `None` when the loop is shutting down).
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
