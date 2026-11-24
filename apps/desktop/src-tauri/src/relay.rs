//! Fil de relais entre le canal `mpsc` du cœur et le Channel de l'interface (ADR 0015).
//!
//! `Started` et le message terminal passent toujours ; entre deux envois, seul le dernier `Step` est gardé, au plus
//! une fois toutes les 50 ms. Les messages ne sont jamais réordonnés.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::summary::ProgressEvent;
use crate::worker::ProgressSink;

pub(crate) const RELAY_INTERVAL: Duration = Duration::from_millis(50);

enum Wake {
    Event(ProgressEvent),
    Due,
    Closed,
}

struct Relay {
    sink: ProgressSink,
    interval: Duration,
    pending: Option<ProgressEvent>,
    last_sent: Option<Instant>,
}

/// Relaie jusqu'à la fermeture du canal, puis envoie le dernier `Step` encore en attente.
pub(crate) fn forward(events: &Receiver<ProgressEvent>, sink: ProgressSink, interval: Duration) {
    let mut relay = Relay {
        sink,
        interval,
        pending: None,
        last_sent: None,
    };
    loop {
        match relay.next(events) {
            Wake::Event(event) if event.is_step() => {
                relay.pending = Some(event);
                relay.flush_if_due();
            }
            Wake::Event(event) => {
                relay.flush();
                relay.send(event);
            }
            Wake::Due => relay.flush_if_due(),
            Wake::Closed => {
                relay.flush();
                return;
            }
        }
    }
}

impl Relay {
    fn next(&self, events: &Receiver<ProgressEvent>) -> Wake {
        let Some(left) = self.time_left() else {
            return events.recv().map_or(Wake::Closed, Wake::Event);
        };
        match events.recv_timeout(left) {
            Ok(event) => Wake::Event(event),
            Err(RecvTimeoutError::Timeout) => Wake::Due,
            Err(RecvTimeoutError::Disconnected) => Wake::Closed,
        }
    }

    /// `None` tant qu'aucun `Step` n'attend : le relais dort jusqu'au prochain message.
    fn time_left(&self) -> Option<Duration> {
        self.pending.as_ref()?;
        Some(self.last_sent.map_or(Duration::ZERO, |at| {
            self.interval.saturating_sub(at.elapsed())
        }))
    }

    fn flush_if_due(&mut self) {
        if self.time_left() == Some(Duration::ZERO) {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if let Some(event) = self.pending.take() {
            self.send(event);
        }
    }

    fn send(&mut self, event: ProgressEvent) {
        (self.sink)(event);
        self.last_sent = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex, PoisonError};
    use std::thread;

    use super::*;

    fn step(done: u32) -> ProgressEvent {
        ProgressEvent::Step {
            done,
            total: 100,
            step: "cleanup.progress.item",
        }
    }

    fn run(events: Vec<ProgressEvent>, interval: Duration) -> Vec<ProgressEvent> {
        let (sender, receiver) = mpsc::channel();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let shared = Arc::clone(&seen);
        let relay = thread::spawn(move || {
            forward(
                &receiver,
                Box::new(move |event| {
                    shared
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(event);
                }),
                interval,
            );
        });
        for event in events {
            sender.send(event).expect("relais vivant");
        }
        drop(sender);
        relay.join().expect("relais terminé");
        seen.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    #[test]
    fn a_burst_keeps_the_ends_and_the_last_step_in_order() {
        let mut burst = vec![ProgressEvent::Started];
        burst.extend((1..=50).map(step));
        burst.push(ProgressEvent::Finished);
        let seen = run(burst, Duration::from_mins(1));
        assert_eq!(seen.first(), Some(&ProgressEvent::Started));
        assert_eq!(seen.last(), Some(&ProgressEvent::Finished));
        let steps: Vec<u32> = seen
            .iter()
            .filter_map(|event| match event {
                ProgressEvent::Step { done, .. } => Some(*done),
                ProgressEvent::Started
                | ProgressEvent::Finished
                | ProgressEvent::Cancelled
                | ProgressEvent::Failed { .. } => None,
            })
            .collect();
        assert_eq!(steps.last(), Some(&50));
        assert!(steps.len() < 50, "coalescé : {steps:?}");
        assert!(steps.is_sorted());
    }

    #[test]
    fn without_a_delay_every_step_passes() {
        let seen = run((1..=5).map(step).collect(), Duration::ZERO);
        assert_eq!(seen, (1..=5).map(step).collect::<Vec<_>>());
    }
}
