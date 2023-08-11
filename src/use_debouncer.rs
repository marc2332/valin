use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use dioxus::prelude::{to_owned, ScopeState, TaskId};
use tokio::time::sleep;

#[derive(Clone)]
pub struct UseDebouncer {
    time: Duration,
    timer: Rc<RefCell<Instant>>,
    current_task_id: Rc<RefCell<Option<TaskId>>>,
}

impl UseDebouncer {
    /// Remove any previous running task
    pub fn cancel(&self, cx: &ScopeState) {
        if let Some(task_id) = self.current_task_id.borrow_mut().take() {
            cx.remove_future(task_id);
        }
    }

    pub fn action(&self, cx: &ScopeState, action: impl FnOnce() + 'static) {
        self.cancel(cx);

        let is_allowed =
            |timer: &Rc<RefCell<Instant>>, time: Duration| timer.borrow().elapsed() > time;
        let restart_timer = |timer: &Rc<RefCell<Instant>>| *timer.borrow_mut() = Instant::now();

        // No need to wait if we are already allowed
        if is_allowed(&self.timer, self.time) {
            // Restart timer and call the action
            restart_timer(&self.timer);
            action();
        } else {
            restart_timer(&self.timer);

            let timer = self.timer.clone();
            let time = self.time;
            let current_task_id = self.current_task_id.clone();
            let my_task_id = Rc::new(RefCell::new(None));

            // Launch a task to check if we are still not allowed
            // when the specified time has passed
            let task_id = cx.push_future({
                to_owned![my_task_id];
                async move {
                    // Wait the specified time
                    sleep(time).await;

                    let is_last_task = *my_task_id.borrow() == *current_task_id.borrow();

                    if is_last_task && is_allowed(&timer, time) {
                        // Restart timer and call the action
                        restart_timer(&timer);
                        action();
                    }
                }
            });

            // Save the new task as the last one
            *my_task_id.borrow_mut() = Some(task_id);
            *self.current_task_id.borrow_mut() = Some(task_id);
        }
    }
}

pub fn use_debouncer(cx: &ScopeState, time: Duration) -> &UseDebouncer {
    cx.use_hook(|| UseDebouncer {
        time,
        timer: Rc::new(RefCell::new(Instant::now())),
        current_task_id: Rc::new(RefCell::new(None)),
    })
}
