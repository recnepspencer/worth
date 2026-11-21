//! Delegate retirement legality to the actual source that owns Native custody.
use super::WorthQueryPrimaryGraph;
use crate::domain_computation::execution_runtime::product_world::{
    RetiringRelationalSource, WorthQueryRelationalSourceOwner,
};
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial;
use std::sync::Mutex;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::runtime::RelationalRuntime;

/// The serving graph and its integration copies each retire their actual
/// local source share. No live accessor escapes the owning post-seal Query.
pub struct PrimaryGraphSourceCustody {
    serving: Option<WorthQueryRelationalSourceOwner>,
    pub invalidation_owner: std::sync::Arc<super::super::SourceInvalidationOwner>,
    retiring: Mutex<Option<PreparedSourceRetirement>>,
}
struct PreparedSourceRetirement {
    backing: Box<SourceRetirementBacking>,
    _funding: Option<ExecutionMemoryReservation>,
}
struct SourceRetirementBacking {
    original: Option<RetiringRelationalSource>,
}
impl PrimaryGraphSourceCustody {
    #[doc(hidden)]
    pub fn invalidation_owner_for_test(
        &self,
    ) -> &std::sync::Arc<super::super::SourceInvalidationOwner> {
        &self.invalidation_owner
    }

    pub fn new(serving: WorthQueryRelationalSourceOwner) -> Self {
        let invalidation_owner = std::sync::Arc::clone(&serving.invalidation_owner)
            .into_any_arc()
            .downcast::<super::super::SourceInvalidationOwner>()
            .expect("primary graph source carries SourceInvalidationOwner");
        Self {
            serving: Some(serving),
            invalidation_owner,
            retiring: Mutex::new(None),
        }
    }
    pub fn preflight_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.serving
            .as_ref()
            .expect("only serving source preflights")
            .preflight_home_retirement(request)?;
        let mut storage = self.retiring.try_lock().map_err(|_| {
            WorthQueryHomeRetirementDenial::Busy(
                crate::domain_computation::primary_graph::WorthQueryHomeRetirementOwner::RelationalSource,
            )
        })?;
        if storage.is_none() {
            let funding = request
                .reserve_retained_memory(std::mem::size_of::<SourceRetirementBacking>() as u64)
                .map_err(WorthQueryHomeRetirementDenial::Execution)?;
            *storage = Some(PreparedSourceRetirement {
                backing: Box::new(SourceRetirementBacking { original: None }),
                _funding: funding,
            });
        }
        Ok(())
    }
    pub fn retain_live(&self) -> WorthQueryRelationalSourceOwner {
        self.serving
            .as_ref()
            .expect("only serving graph issues source aliases")
            .clone()
    }
    pub fn advance_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        let storage = self
            .retiring
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_mut()
            .expect("original source storage was prepaid before World seal");
        if let Some(serving) = self.serving.take() {
            storage.backing.original = Some(serving.into_home_retirement());
        }
        storage
            .backing
            .original
            .as_mut()
            .expect("same original source retirement")
            .advance(request, native)
            .map_err(Into::into)
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
    pub fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        self.source_owner.advance_retirement(request, native)
    }

    pub fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.source_owner.preflight_home_retirement(request)
    }
}

impl super::WorthQueryPrimaryGraphIntegrationHandle {
    pub fn preflight_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.source_owner.preflight_home_retirement(request)
    }

    pub fn advance_home_retirement(
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
