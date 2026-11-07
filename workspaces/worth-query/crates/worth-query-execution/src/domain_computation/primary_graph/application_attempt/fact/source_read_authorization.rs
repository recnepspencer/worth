//! Original completed Source READ support, independent of structural Source data.
use crate::domain_computation::{
    authorization::{
        SourceReadSupport, WorthQueryInstalledAuthorizationRegistry,
        WorthQueryRetainedAuthorizationDecisionFacts,
    },
    primary_graph::{
        invariant_projection::ConsumedOutputVerificationStop, source_facts::SourceFactsAllocation,
    },
    provider_session::{WorthQueryCompletedSourceRead, WorthQueryGraphWorkSessionIdentity},
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation;
use worth_relational::facade::{
    identity::EntityId, mvcc::CompanionPreflightStop, runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct SourceReadAuthorizationKey {
    pub source: [u8; 32],
    pub support: [u8; 32],
    pub principal: EntityId,
    pub scope: EntityId,
}

pub(in crate::domain_computation) struct SourceReadAuthorization {
    key: SourceReadAuthorizationKey,
    binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
    session: WorthQueryGraphWorkSessionIdentity,
    support: SourceReadSupport,
    identity: [u8; 32],
}

impl SourceReadAuthorization {
    /// Only successful Source completion supplies the actual terminal and its
    /// exact bound query. Descriptive query identities cannot create custody.
    pub(in crate::domain_computation::primary_graph) fn complete(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        terminal: &WorthQueryCompletedSourceRead,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        source: [u8; 32],
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Option<Arc<Self>>, CompanionPreflightStop> {
        if !authorization.belongs_to_session(terminal.session_identity()) {
            return Ok(None);
        }
        let support = SourceReadSupport::retain(execution, authorization, registry, allocation)?;
        SourceFactsAllocation::work(execution, 256).map_err(CompanionPreflightStop::Execution)?;
        let mut digest = Sha256::new();
        digest.update(b"worth-query:source-read-authorization:v1");
        digest.update(source);
        for entity in [terminal.principal(), terminal.scope()] {
            digest.update(entity.partition_value().to_be_bytes());
            digest.update(entity.local_slot_value().to_be_bytes());
            digest.update(entity.generation_value().to_be_bytes());
        }
        if !support.visit_support(&mut |bytes| {
            SourceFactsAllocation::work(execution, bytes.len() as u64)
                .map_err(CompanionPreflightStop::Execution)?;
            digest.update(bytes);
            Ok::<_, CompanionPreflightStop>(())
        })? {
            return Ok(None);
        }
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let identity = digest.finalize().into();
        Ok(Some(Arc::new(Self {
            key: SourceReadAuthorizationKey {
                source,
                support: identity,
                principal: terminal.principal(),
                scope: terminal.scope(),
            },
            binding: Some(binding),
            session: terminal.session_identity(),
            support,
            identity,
        })))
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &SourceReadAuthorizationKey {
        &self.key
    }
    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(in crate::domain_computation::primary_graph) fn matches_binding(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBindingObservation,
    ) -> bool {
        self.binding.as_ref() == Some(binding)
    }
    pub(in crate::domain_computation::primary_graph) fn validates_terminal(
        &self,
        terminal: &WorthQueryCompletedSourceRead,
    ) -> bool {
        self.binding.is_some()
            && self.session == terminal.session_identity()
            && self.support.belongs_to_session(self.session)
            && self.key.principal == terminal.principal()
            && self.key.scope == terminal.scope()
    }
    pub(in crate::domain_computation::primary_graph) fn validate_current(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        self.support.validate_current(execution, runtime, snapshot).map_err(crate::domain_computation::primary_graph::source_facts::map_source_authorization_currentness)
    }
    pub(in crate::domain_computation::primary_graph) fn same_truth<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(1)?;
        if self.key != other.key || self.binding != other.binding || self.identity != other.identity
        {
            return Ok(false);
        }
        self.support
            .same_conventional_support(&other.support, admit)
    }
    pub(in crate::domain_computation::primary_graph) fn visit_loci<E>(
        &self,
        visit: &mut impl FnMut(
            crate::domain_computation::authorization::SourceAuthorizationLocus<'_>,
        ) -> Result<(), E>,
    ) -> Result<bool, E> {
        self.support.visit_loci(visit)
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

impl SourceReadAuthorization {
    /// A successful independent operation READ has its own conventional C
    /// support. It cannot validate as a query completion or mint a query receipt.
    pub(in crate::domain_computation) fn retain_operation(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: worth_query_installation::facade::InstalledOperationView<'_>,
        issuer: &crate::domain_computation::authorization::BorrowedReadOperationAdmission<'_>,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let allocation =
            SourceFactsAllocation::open(execution).map_err(CompanionPreflightStop::Execution)?;
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let support = SourceReadSupport::retain(execution, authorization, registry, &allocation)?;
        let mut digest = Sha256::new();
        digest.update(b"worth-query:operation-read-authorization:v1");
        digest.update(operation.authority_identity_bytes());
        for entity in [issuer.principal, issuer.scope] {
            digest.update(entity.partition_value().to_be_bytes());
            digest.update(entity.local_slot_value().to_be_bytes());
            digest.update(entity.generation_value().to_be_bytes());
        }
        let conventional = support.visit_support(&mut |bytes| {
            SourceFactsAllocation::work(execution, bytes.len() as u64)
                .map_err(CompanionPreflightStop::Execution)?;
            digest.update(bytes);
            Ok::<_, CompanionPreflightStop>(())
        })?;
        if !conventional {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let identity = digest.finalize().into();
        Ok(Arc::new(Self {
            key: SourceReadAuthorizationKey {
                source: operation.authority_identity_bytes(),
                support: identity,
                principal: issuer.principal,
                scope: issuer.scope,
            },
            binding: None,
            session: issuer.session,
            support,
            identity,
        }))
    }
}
