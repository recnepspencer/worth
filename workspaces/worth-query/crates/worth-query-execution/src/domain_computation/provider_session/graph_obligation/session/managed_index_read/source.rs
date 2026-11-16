//! Exact completed READ custody for the two real Source truth postures.
use super::*;
use crate::domain_computation::provider_session::WorthQueryAppliedSourceRead;

pub(in crate::domain_computation) enum ManagedIndexReadSource {
    Positioned(Arc<WorthQueryCompletedSourceRead>),
    Applied(Arc<WorthQueryAppliedSourceRead>),
}
impl ManagedIndexReadSource {
    pub(super) fn admits_parent<Stop>(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        match self {
            Self::Positioned(source) => source.admits_parent(parent, admit, overflow),
            Self::Applied(source) => source.admits_parent(parent, admit, overflow),
        }
    }
    pub(super) fn runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        match self {
            Self::Positioned(source) => source.runtime(),
            Self::Applied(source) => source.runtime(),
        }
    }
    pub(super) fn principal(&self) -> EntityId {
        match self {
            Self::Positioned(source) => source.principal(),
            Self::Applied(source) => source.principal(),
        }
    }
    pub(super) fn scope(&self) -> EntityId {
        match self {
            Self::Positioned(source) => source.scope(),
            Self::Applied(source) => source.scope(),
        }
    }
    pub(super) fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        match self {
            Self::Positioned(source) => source.binding(),
            Self::Applied(source) => source.binding(),
        }
    }
    pub(super) fn obligation(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        match self {
            Self::Positioned(source) => source.obligation(),
            Self::Applied(source) => source.obligation(),
        }
    }
    pub(super) fn matches_positioned(&self, other: &Arc<WorthQueryCompletedSourceRead>) -> bool {
        matches!(self, Self::Positioned(source) if Arc::ptr_eq(source, other))
    }
    pub(super) fn matches_applied(&self, other: &Arc<WorthQueryAppliedSourceRead>) -> bool {
        matches!(self, Self::Applied(source) if Arc::ptr_eq(source, other))
    }
}
