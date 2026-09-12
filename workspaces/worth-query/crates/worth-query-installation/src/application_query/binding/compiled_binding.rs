use std::sync::Arc;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingDescriptor, ApplicationQueryScopeContract,
};

use super::{WorthQueryCompiledApplicationQuery, WorthQueryInstalledApplicationQueryLimits};

pub(crate) struct WorthQueryCompiledApplicationQueryBinding {
    descriptor: ApplicationQueryBindingDescriptor,
    query: Arc<WorthQueryCompiledApplicationQuery>,
    limits: WorthQueryInstalledApplicationQueryLimits,
}

impl WorthQueryCompiledApplicationQueryBinding {
    pub(crate) fn new(
        descriptor: ApplicationQueryBindingDescriptor,
        query: Arc<WorthQueryCompiledApplicationQuery>,
        limits: WorthQueryInstalledApplicationQueryLimits,
    ) -> Self {
        Self {
            descriptor,
            query,
            limits,
        }
    }

    pub(crate) fn descriptor(&self) -> &ApplicationQueryBindingDescriptor {
        &self.descriptor
    }

    pub(crate) fn query(&self) -> Arc<WorthQueryCompiledApplicationQuery> {
        Arc::clone(&self.query)
    }

    pub(crate) fn scope(&self) -> &ApplicationQueryScopeContract {
        self.descriptor.scope()
    }

    pub(crate) const fn limits(&self) -> WorthQueryInstalledApplicationQueryLimits {
        self.limits
    }
}
