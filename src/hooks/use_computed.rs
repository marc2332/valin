use std::{cell::RefCell, rc::Rc};

use freya::prelude::use_hook;

pub struct Memoized<T, D> {
    pub value: T,
    pub deps: D,
}

pub fn use_computed<T: 'static, D>(deps: &D, init: impl Fn(&D) -> T) -> Rc<RefCell<Memoized<T, D>>>
where
    D: PartialEq + 'static + ToOwned<Owned = D>,
    D::Owned: PartialEq,
{
    let memo = use_hook(|| {
        Rc::new(RefCell::new(Memoized {
            value: init(deps),
            deps: deps.to_owned(),
        }))
    });

    let deps_have_changed = &memo.borrow().deps != deps;

    if deps_have_changed {
        memo.borrow_mut().value = init(deps);
        memo.borrow_mut().deps = deps.to_owned();
    }

    memo
}
