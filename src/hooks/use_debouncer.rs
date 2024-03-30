use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::{dioxus_core::use_hook, prelude::spawn};
use futures::channel::mpsc::UnboundedSender as Sender;
use futures::StreamExt;

pub type DebouncedCallback = Box<dyn FnOnce()>;

#[derive(Clone)]
pub struct UseDebouncer {
    sender: Rc<RefCell<Sender<DebouncedCallback>>>,
}

impl PartialEq for UseDebouncer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.sender, &other.sender)
    }
}

impl UseDebouncer {
    pub fn action(&self, action: impl FnOnce() + 'static) {
        self.sender
            .borrow_mut()
            .unbounded_send(Box::new(action))
            .ok();
    }
}

pub fn use_debouncer(time: Duration) -> UseDebouncer {
    use_hook(|| {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let debouncer = UseDebouncer {
            sender: Rc::new(RefCell::new(sender)),
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
