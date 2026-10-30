use std::sync::Arc;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingDescriptor, ApplicationQueryScopeContract,
};

use super::{WorthQueryCompiledApplicationQuery, WorthQueryInstalledApplicationQueryLimits};

pub(crate) struct WorthQueryCompiledApplicationQueryBinding {
    descriptor: ApplicationQueryBindingDescriptor,
    query: Arc<WorthQueryCompiledApplicationQuery>,
    limits: WorthQueryInstalledApplicationQueryLimits,
    inspection: Option<Arc<crate::application_query::inspection::CompiledInspectionMeaning>>,
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
            inspection: None,
        }
    }

    pub(crate) fn descriptor(&self) -> &ApplicationQueryBindingDescriptor {
        &self.descriptor
    }

    pub(crate) fn query(&self) -> Arc<WorthQueryCompiledApplicationQuery> {
        Arc::clone(&self.query)
    }

    pub(crate) fn query_identity(
        &self,
    ) -> &crate::application_query::WorthQueryInstalledApplicationQueryIdentity {
        self.query.identity()
    }

    pub(crate) fn scope(&self) -> &ApplicationQueryScopeContract {
        self.descriptor.scope()
    }

    pub(crate) const fn limits(&self) -> WorthQueryInstalledApplicationQueryLimits {
        self.limits
    }

    /// Called once while the catalog still uniquely owns this unpublished binding.
    pub(super) fn finish_inspection(
        &mut self,
        inspection: crate::application_query::inspection::CompiledInspectionMeaning,
    ) {
        assert!(
            self.inspection.is_none(),
            "inspection meaning is finished once"
        );
        self.inspection = Some(Arc::new(inspection));
    }

    pub(super) fn inspection_meaning(
        &self,
    ) -> Option<Arc<crate::application_query::inspection::CompiledInspectionMeaning>> {
        self.inspection.as_ref().map(Arc::clone)
    }
}
