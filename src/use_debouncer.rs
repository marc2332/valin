use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::ScopeState;
use futures::channel::mpsc::UnboundedSender as Sender;
use futures::StreamExt;

#[derive(Clone)]
pub struct UseDebouncer {
    sender: Rc<RefCell<Sender<Box<dyn FnOnce() -> ()>>>>,
}

impl UseDebouncer {
    pub fn action(&self, action: impl FnOnce() + 'static) {
        self.sender
            .borrow_mut()
            .unbounded_send(Box::new(action))
            .ok();
    }
}

pub fn use_debouncer(cx: &ScopeState, time: Duration) -> &UseDebouncer {
    cx.use_hook(|| {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let debouncer = UseDebouncer {
            sender: Rc::new(RefCell::new(sender)),
        };

        let mut debounced = debounced::debounced(receiver, time);

        cx.push_future(async move {
            loop {
                if let Some(cb) = debounced.next().await {
                    cb();
                }
            }
        });

        debouncer
    })
}
