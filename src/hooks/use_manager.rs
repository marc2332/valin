use std::{
    cell::{Ref, RefCell, RefMut}, collections::HashMap, ops::{Deref, DerefMut}, rc::Rc, sync::Arc
};

use dioxus::{dioxus_core::{schedule_update_any, use_hook}, prelude::{
    current_scope_id, use_context, use_context_provider, Coroutine, ScopeId
}};

pub use crate::editor_manager::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SubscriptionModel {
    All,
    Tab {
        panel_index: usize,
        editor_index: usize,
    },
}

impl SubscriptionModel {
    pub fn follow_tab(panel: usize, editor: usize) -> Self {
        Self::Tab {
            panel_index: panel,
            editor_index: editor,
        }
    }
}

pub type SharedEditorManager = Rc<EditorManagerInner>;

pub fn use_init_manager(
    lsp_status_coroutine: &Coroutine<(String, String)>,
) -> SharedEditorManager {
    use_context_provider(|| {
        Rc::new(EditorManagerInner::new(
            EditorManager::new(lsp_status_coroutine.clone()),
        ))
    })
}

pub fn use_manager(model: SubscriptionModel) -> UseManager {
    let manager = use_context::<SharedEditorManager>();

    let manager = use_hook(|| {
        let mut manager = manager.as_ref().clone();
        manager.scope = current_scope_id().unwrap();
        UseManager::new(manager, model.clone())
    });

    manager.update_model_if_necessary(model);

    manager
}

#[derive(Clone)]
pub struct EditorManagerInner {
    pub subscribers: Rc<RefCell<HashMap<ScopeId, SubscriptionModel>>>,
    value: Rc<RefCell<EditorManager>>,
    scheduler: Arc<dyn Fn(ScopeId) + Send + Sync>,
    scope: ScopeId,
}

impl Drop for EditorManagerInner {
    fn drop(&mut self) {
        self.subscribers.borrow_mut().remove(&self.scope);
    }
}

#[derive(Clone)]
pub struct UseManager {
    inner: SharedEditorManager,
}

impl PartialEq for UseManager {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl UseManager {
    pub fn new(inner: EditorManagerInner, model: SubscriptionModel) -> Self {
        inner.subscribers.borrow_mut().insert(inner.scope, model);
        Self {
            inner: Rc::new(inner),
        }
    }

    fn update_model_if_necessary(&self, model: SubscriptionModel) {
        let mut subs = self.inner.subscribers.borrow_mut();
        let entry = subs.get_mut(&self.inner.scope);

        if let Some(entry) = entry {
            if entry != &model {
                *entry = model
            }
        }
    }

    pub fn global_write(&self) -> EditorManagerInnerGuard {
        self.inner.global_write()
    }

    pub fn write(&self) -> EditorManagerInnerGuard {
        self.inner.write()
    }

    pub fn current(&self) -> Ref<EditorManager> {
        self.inner.current()
    }
}

pub struct EditorManagerInnerGuard<'a> {
    model: SubscriptionModel,
    pub subscribers: Rc<RefCell<HashMap<ScopeId, SubscriptionModel>>>,
    value: RefMut<'a, EditorManager>,
    scheduler: Arc<dyn Fn(ScopeId) + Send + Sync>,
}

impl EditorManagerInner {
    pub fn new(value: EditorManager) -> Self {
        Self {
            subscribers: Rc::new(RefCell::new(HashMap::from([(
                current_scope_id().unwrap(),
                SubscriptionModel::All,
            )]))),
            value: Rc::new(RefCell::new(value.clone())),
            scheduler: schedule_update_any(),
            scope: current_scope_id().unwrap(),
        }
    }

    pub fn global_write(&self) -> EditorManagerInnerGuard {
        EditorManagerInnerGuard {
            model: SubscriptionModel::All,
            subscribers: self.subscribers.clone(),
            value: self.value.borrow_mut(),
            scheduler: self.scheduler.clone(),
        }
    }

    pub fn write(&self) -> EditorManagerInnerGuard {
        let model = {
            let subscribers = self.subscribers.borrow();
            subscribers.get(&self.scope).unwrap().clone()
        };
        EditorManagerInnerGuard {
            model,
            subscribers: self.subscribers.clone(),
            value: self.value.borrow_mut(),
            scheduler: self.scheduler.clone(),
        }
    }

    pub fn current(&self) -> Ref<EditorManager> {
        self.value.borrow()
    }
}

impl Drop for EditorManagerInnerGuard<'_> {
    fn drop(&mut self) {
        for (scope_id, scope_model) in self.subscribers.borrow().iter() {
            if scope_model == &self.model {
                (self.scheduler)(*scope_id)
            }
        }
    }
}

impl<'a> Deref for EditorManagerInnerGuard<'a> {
    type Target = RefMut<'a, EditorManager>;

    fn deref(&self) -> &RefMut<'a, EditorManager> {
        &self.value
    }
}

impl<'a> DerefMut for EditorManagerInnerGuard<'a> {
    fn deref_mut(&mut self) -> &mut RefMut<'a, EditorManager> {
        &mut self.value
    }
}
