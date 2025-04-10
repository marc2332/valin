use dioxus::prelude::*;
use futures::{
    channel::mpsc::{self, UnboundedSender as Sender},
    StreamExt,
};
use std::time::Duration;

/// The interface for calling a debounce.
///
/// See [`use_debounce`] for more information.
pub struct UseDebounce<T: 'static> {
    sender: Signal<Sender<T>>,
    cancel: Signal<bool>,
}

impl<T> UseDebounce<T> {
    /// Will start the debounce countdown, resetting it if already started.
    pub fn action(&mut self, data: T) {
        self.cancel.set(false);
        self.sender.write().unbounded_send(data).ok();
    }

    pub fn cancel(&mut self) {
        self.cancel.set(true);
    }
}

// Manually implement Clone, Copy, and PartialEq as #[derive] thinks that T needs to implement these (it doesn't).

impl<T> Clone for UseDebounce<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UseDebounce<T> {}

impl<T> PartialEq for UseDebounce<T> {
    fn eq(&self, other: &Self) -> bool {
        self.sender == other.sender
    }
}

pub fn use_debounce<T>(time: Duration, cb: impl FnOnce(T) + Copy + 'static) -> UseDebounce<T> {
    use_hook(|| {
        let (sender, mut receiver) = mpsc::unbounded();
        let mut cancel = Signal::new(false);
        let debouncer = UseDebounce {
            sender: Signal::new(sender),
            cancel,
        };

        spawn(async move {
            let mut current_task: Option<Task> = None;

            loop {
                if let Some(data) = receiver.next().await {
                    if let Some(task) = current_task.take() {
                        task.cancel();
                    }

                    current_task = Some(spawn(async move {
                        #[cfg(not(target_family = "wasm"))]
                        tokio::time::sleep(time).await;

                        #[cfg(target_family = "wasm")]
                        gloo_timers::future::sleep(time).await;

                        if *cancel.peek() {
                            cancel.set(false);
                            return;
                        }

                        cb(data);
                    }));
                }
            }
        });

        debouncer
    })
}
