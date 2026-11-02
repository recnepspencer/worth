//! Delegate retirement legality to the actual source that owns Native custody.
use super::WorthQueryPrimaryGraph;
use crate::domain_computation::execution_runtime::product_world::{
    RetiringRelationalSource, WorthQueryRelationalSourceOwner,
};
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial;
use worth_relational::facade::runtime::RelationalRuntime;

/// The serving graph and its integration copies each retire their actual
/// local source share. No live accessor escapes the owning post-seal Query.
pub(in crate::domain_computation::primary_graph) struct PrimaryGraphSourceCustody {
    serving: Option<WorthQueryRelationalSourceOwner>,
    retiring: Option<RetiringRelationalSource>,
}
impl PrimaryGraphSourceCustody {
    pub(super) fn new(serving: WorthQueryRelationalSourceOwner) -> Self {
        Self {
            serving: Some(serving),
            retiring: None,
        }
    }
    pub(super) fn retain_live(&self) -> WorthQueryRelationalSourceOwner {
        self.serving
            .as_ref()
            .expect("only serving graph issues source aliases")
            .clone()
    }
    pub(in crate::domain_computation::primary_graph) fn advance_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        if let Some(serving) = self.serving.take() {
            self.retiring = Some(serving.into_home_retirement());
        }
        self.retiring
            .as_mut()
            .expect("same original source retirement")
            .advance(request, native)
    }
}
impl Clone for PrimaryGraphSourceCustody {
    fn clone(&self) -> Self {
        Self::new(self.retain_live())
    }
}
impl std::ops::Deref for PrimaryGraphSourceCustody {
    type Target = WorthQueryRelationalSourceOwner;
    fn deref(&self) -> &Self::Target {
        self.serving
            .as_ref()
            .expect("retired Query exposes no serving graph")
    }
}
impl std::ops::DerefMut for PrimaryGraphSourceCustody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.serving
            .as_mut()
            .expect("retired Query exposes no serving graph")
    }
}

impl WorthQueryPrimaryGraph {
    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        self.source_owner.advance_retirement(request, native)
    }

    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.source_owner.preflight_home_retirement(request)
    }
}

impl super::WorthQueryPrimaryGraphIntegrationHandle {
    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        self.source_owner.advance_retirement(request, native)
    }
}
