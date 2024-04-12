use std::time::Duration;

use dioxus::{dioxus_core::use_hook, prelude::spawn};
use freya::prelude::{Signal, Writable};
use futures::channel::mpsc::UnboundedSender as Sender;
use futures::StreamExt;

pub type DebouncedCallback = Box<dyn FnOnce()>;

#[derive(Clone, PartialEq, Copy)]
pub struct UseDebouncer {
    sender: Signal<Sender<DebouncedCallback>>,
}

impl UseDebouncer {
    pub fn action(&mut self, action: impl FnOnce() + 'static) {
        self.sender.write().unbounded_send(Box::new(action)).ok();
    }
}

pub fn use_debouncer(time: Duration) -> UseDebouncer {
    use_hook(|| {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let debouncer = UseDebouncer {
            sender: Signal::new(sender),
        };

        let mut debounced = debounced::debounced(receiver, time);

        spawn(async move {
            loop {
                if let Some(cb) = debounced.next().await {
                    cb();
                }
            }
        });

        debouncer
    })
}
