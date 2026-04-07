#[path = "../src/support/paths.rs"]
pub mod support_paths;

mod support {
    pub use crate::support_paths as paths;
}

#[path = "../src/service/runtime_state_store.rs"]
mod runtime_state_store;

use std::collections::BTreeSet;

use runtime_state_store::{PersistedRuntimeState, RuntimeStateStore};
use support::paths::AppPaths;
use tempfile::tempdir;

#[test]
fn runtime_state_store_round_trips_last_active_rules() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let store = RuntimeStateStore::new(paths);

    let snapshot = PersistedRuntimeState {
        last_active_rule_ids: BTreeSet::from(["rule-1".to_string(), "rule-2".to_string()]),
    };

    store.save(&snapshot).expect("save runtime state");
    let restored = store.load().expect("load runtime state");

    assert_eq!(restored, snapshot);
}
