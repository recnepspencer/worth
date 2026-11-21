//! Original completed Source READ support, independent of structural Source data.
use crate::domain_computation::{
    primary_graph::{
        invariant_projection::ConsumedOutputVerificationStop, source_facts::SourceFactsAllocation,
    },
    provider_session::{WorthQueryCompletedSourceRead, WorthQueryGraphWorkSessionIdentity},
};
use crate::ports::{SourceAuthorizationLocus, SourceReadSupportPort};
pub mod candidate_support;
use std::any::Any;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation;
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceReadAuthorizationKey {
    pub source: [u8; 32],
    pub support: [u8; 32],
    pub principal: EntityId,
    pub scope: EntityId,
}

pub struct SourceReadAuthorization {
    pub key: SourceReadAuthorizationKey,
    pub binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
    pub session: WorthQueryGraphWorkSessionIdentity,
    pub support: Box<dyn SourceReadSupportPort>,
    pub identity: [u8; 32],
    pub applied: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity>,
}

impl SourceReadAuthorization {
    pub fn from_parts(
        key: SourceReadAuthorizationKey,
        binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
        session: WorthQueryGraphWorkSessionIdentity,
        support: Box<dyn SourceReadSupportPort>,
        identity: [u8; 32],
        applied: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity>,
    ) -> Self {
        Self {
            key,
            binding,
            session,
            support,
            identity,
            applied,
        }
    }

    pub fn key(&self) -> &SourceReadAuthorizationKey {
        &self.key
    }

    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub fn binding(&self) -> Option<&WorthQueryInstalledApplicationQueryBindingObservation> {
        self.binding.as_ref()
    }

    pub fn session(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }

    pub fn support(&self) -> &dyn SourceReadSupportPort {
        &*self.support
    }

    pub fn matches_binding(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBindingObservation,
    ) -> bool {
        self.binding.as_ref() == Some(binding)
    }

    pub fn validates_terminal(&self, terminal: &WorthQueryCompletedSourceRead) -> bool {
        self.applied.is_none()
            && self.binding.is_some()
            && self.session == terminal.session_identity()
            && self.support.belongs_to_session(self.session)
            && self.key.principal == terminal.principal()
            && self.key.scope == terminal.scope()
    }

    pub fn validate_current(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        self.support.validate_current(execution, runtime, snapshot)
    }

    pub fn same_truth<E: 'static>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(1)?;
        if self.key != other.key || self.binding != other.binding || self.identity != other.identity
        {
            return Ok(false);
        }
        let mut erased = |units: u64| admit(units).map_err(|err| Box::new(err) as Box<dyn Any>);
        self.support
            .same_conventional_support_dyn(&*other.support, &mut erased)
            .map_err(|err| *err.downcast::<E>().expect("matching error type"))
    }

    pub fn visit_loci<E: 'static>(
        &self,
        visit: &mut impl FnMut(SourceAuthorizationLocus<'_>) -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut erased = |locus: SourceAuthorizationLocus<'_>| {
            visit(locus).map_err(|err| Box::new(err) as Box<dyn Any>)
        };
        self.support
            .visit_loci_dyn(&mut erased)
            .map_err(|err| *err.downcast::<E>().expect("matching error type"))
    }
}

impl PartialEq for SourceReadAuthorization {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for SourceReadAuthorization {}

impl std::fmt::Debug for SourceReadAuthorization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceReadAuthorization")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}
