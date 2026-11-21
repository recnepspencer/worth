//! Runtime constructors on `SourceReadAuthorization` bridging `SourceReadSupport`.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::authorization::{
    BorrowedReadOperationAdmission, SourceReadSupport, WorthQueryAdmittedApplicationOperation,
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::application_attempt::{
    SourceReadAuthorization, SourceReadAuthorizationKey,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use crate::domain_computation::provider_session::{
    WorthQueryCompletedSourceRead, WorthQueryGraphWorkSessionIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation;

struct OperationReadOrigin {
    authority: [u8; 32],
    session: WorthQueryGraphWorkSessionIdentity,
    principal: EntityId,
    scope: EntityId,
}

pub(in crate::domain_computation) trait SourceReadAuthorizationRuntimeExt: Sized {
    #[allow(clippy::too_many_arguments)]
    fn complete(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        terminal: &WorthQueryCompletedSourceRead,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        source: [u8; 32],
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Option<Arc<Self>>, CompanionPreflightStop>;

    fn retain_operation(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: worth_query_installation::facade::InstalledOperationView<'_>,
        issuer: &BorrowedReadOperationAdmission<'_>,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
    ) -> Result<Arc<Self>, CompanionPreflightStop>;

    fn consume_operation<S, O, I, E>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, E>,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
    ) -> Result<Arc<Self>, CompanionPreflightStop>;
}

impl SourceReadAuthorizationRuntimeExt for SourceReadAuthorization {
    fn complete(
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
            support: Box::new(support),
            identity,
            applied: None,
        })))
    }

    fn retain_operation(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: worth_query_installation::facade::InstalledOperationView<'_>,
        issuer: &BorrowedReadOperationAdmission<'_>,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        operation_support(
            execution,
            OperationReadOrigin {
                authority: operation.authority_identity_bytes(),
                session: issuer.session,
                principal: issuer.principal,
                scope: issuer.scope,
            },
            authorization,
            registry,
        )
    }

    fn consume_operation<S, O, I, E>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, E>,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let authority = *admission.operation_authority_identity_bytes();
        let session = admission.graph_work_session_identity();
        let principal = admission.graph_work_principal_entity_id();
        let scope = admission.scope_entity_id();
        let authorization = admission
            .into_read_authorization()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        if !authorization.belongs_to_session(session) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        operation_support(
            execution,
            OperationReadOrigin {
                authority,
                session,
                principal,
                scope,
            },
            authorization,
            registry,
        )
    }
}

fn operation_support(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    origin: OperationReadOrigin,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
) -> Result<Arc<SourceReadAuthorization>, CompanionPreflightStop> {
    let OperationReadOrigin {
        authority,
        session,
        principal,
        scope,
    } = origin;
    let allocation =
        SourceFactsAllocation::open(execution).map_err(CompanionPreflightStop::Execution)?;
    allocation
        .claim_arc::<SourceReadAuthorization>(execution)
        .map_err(CompanionPreflightStop::Execution)?;
    let support = SourceReadSupport::retain(execution, authorization, registry, &allocation)?;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:operation-read-authorization:v1");
    digest.update(authority);
    for entity in [principal, scope] {
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
    Ok(Arc::new(SourceReadAuthorization {
        key: SourceReadAuthorizationKey {
            source: authority,
            support: identity,
            principal,
            scope,
        },
        binding: None,
        session,
        support: Box::new(support),
        identity,
        applied: None,
    }))
}
