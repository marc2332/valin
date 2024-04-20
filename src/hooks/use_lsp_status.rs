use std::collections::HashMap;

use freya::prelude::*;
use tokio::sync::mpsc;

pub type LspStatuses = Signal<HashMap<String, String>>;
pub type LspStatusSender = mpsc::UnboundedSender<(String, String)>;

pub fn use_lsp_status() -> (LspStatuses, LspStatusSender) {
    let mut statuses = use_signal::<HashMap<String, String>>(HashMap::default);

    let sender = use_hook(move || {
        let (tx, mut rx) = mpsc::unbounded_channel();

        spawn(async move {
            while let Some((name, val)) = rx.recv().await {
                statuses.write().insert(name, val);
            }
        });

        tx
    });

    (statuses, sender)
}
