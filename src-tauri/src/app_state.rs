use crate::service::rule_store::RuleStore;
use crate::support::paths::AppPaths;

#[derive(Clone)]
pub struct AppState {
    paths: AppPaths,
    rule_store: RuleStore,
}

impl AppState {
    pub fn new(paths: AppPaths, rule_store: RuleStore) -> Self {
        Self { paths, rule_store }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn rule_store(&self) -> &RuleStore {
        &self.rule_store
    }
}
