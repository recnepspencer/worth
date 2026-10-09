//! Exact-basis Query completion candidate for a retained authenticated occurrence.

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    mvcc::{PreparedRelationalCommitCandidate, RelationalTransactionIntent},
    transactions::{CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch},
};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;

mod history_read;
mod preparation_denial;
mod read;
mod transport;
pub(in crate::domain_computation::primary_graph) use read::{
    WorthQueryCanonicalCompletionRow, WorthQueryCompletionProvenance,
    WorthQueryInboundCompletionReadDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryInboundCompletionPreparationDenial {
    ExecutionDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    },
    ExecutionControlStopped {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    },
    OriginalOutboxNotAnEntity,
    ForeignOrStaleBasis,
    AllocationDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_execution::ExecutionAllocationDenialKind,
        requested_payload_bytes: Option<u64>,
    },
    StagingUnavailable,
    StagingAllocationDenied {
        kind: worth_execution::ExecutionAllocationDenialKind,
        requested_payload_bytes: Option<u64>,
    },
    StagingCardinalityOverflow,
    StagingInputDirectoryAllocationDenied {
        requested_batches: usize,
    },
    ValidationUnavailable,
    PreparationUnavailable,
    SnapshotUnavailable,
    IndexPreparationUnavailable,
    InstalledBindingMismatch,
    DispatchOwnerMismatch,
}

impl WorthQueryPrimaryGraphProvider {
    /// Prepares one Relational mutation at the exact World-admitted basis.
    /// The completion marker and occurrence consumption facts share one entity
    /// in the candidate World will publish; this does not independently commit.
    pub(in crate::domain_computation::primary_graph) fn prepare_inbound_completion_candidate(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        accepted: &WorthQueryAcceptedInboundOccurrence,
    ) -> Result<PreparedRelationalCommitCandidate, WorthQueryInboundCompletionPreparationDenial>
    {
        use WorthQueryInboundCompletionPreparationDenial as Denial;
        let RecordRef::Entity(original_entity) = accepted.owner().record_ref() else {
            return Err(Denial::OriginalOutboxNotAnEntity);
        };
        let claims = accepted.claims();
        let record = accepted.owner().record();
        let layout = self.graph.layout.provider_inbound_completion();
        let correlation = hex(record.correlation().digest().bytes());
        let fields = BTreeMap::from([
            (layout.correlation.clone(), string(correlation.clone())),
            (
                layout.family.clone(),
                string(record.correlation_family().as_str()),
            ),
            (layout.operation.clone(), string(accepted.operation())),
            (layout.audience.clone(), string(&claims.audience)),
            (layout.source.clone(), string(&claims.source_identity)),
            (
                layout.expires_at.clone(),
                AspectValue::UInt64(claims.expires_at_unix_seconds),
            ),
            (
                layout.key_epoch.clone(),
                AspectValue::UInt64(claims.key_epoch),
            ),
            (
                layout.message.clone(),
                string(hex(&claims.message_identity)),
            ),
            (
                layout.protocol.clone(),
                string(claims.protocol_identity.as_str()),
            ),
            (
                layout.version.clone(),
                AspectValue::UInt64(u64::from(claims.protocol_version.get())),
            ),
            (
                layout.meaning_digest.clone(),
                string(hex(accepted.signed_meaning_digest())),
            ),
            (layout.payload.clone(), string(hex(&claims.payload))),
            (
                layout.original_commit.clone(),
                AspectValue::UInt64(accepted.owner().commit_reference().commit_id.0),
            ),
            (
                layout.original_branch.clone(),
                string(&accepted.owner().commit_reference().branch_id.0),
            ),
            (
                layout.original_entity.clone(),
                string(format!(
                    "{}:{}:{}",
                    original_entity.partition_id.0,
                    original_entity.local_slot.0,
                    original_entity.generation.0
                )),
            ),
            (
                layout.original_incarnation.clone(),
                AspectValue::UInt64(
                    accepted
                        .owner()
                        .committed_product_publication()
                        .product_incarnation()
                        .ordinal(),
                ),
            ),
            (layout.terminal.clone(), string("consumed-completed")),
            (
                layout.provenance_kind.clone(),
                string("authenticated-inbound"),
            ),
            (layout.transport_attempt.clone(), string("")),
            (layout.transport_observation.clone(), string("")),
        ]);
        self.prepare_completion_fields(basis, *original_entity, &correlation, fields)
    }

    fn prepare_completion_fields(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        original_entity: worth_relational::facade::identity::EntityId,
        correlation: &str,
        fields: BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
    ) -> Result<PreparedRelationalCommitCandidate, WorthQueryInboundCompletionPreparationDenial>
    {
        use WorthQueryInboundCompletionPreparationDenial as Denial;
        let layout = self.graph.layout.provider_inbound_completion();
        let intent = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: original_entity.partition_id,
            kind_id: layout.kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw(format!(
                "worth-query-inbound-completion:{correlation}"
            )),
            fields: worth_relational::facade::transactions::AspectFieldPatch::from(fields),
        }));
        self.graph.with_runtime_mut(|runtime| {
            let before = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|_| Denial::SnapshotUnavailable)?;
            let result = (|| {
                let mut transaction = runtime
                    .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
                    .map_err(|_| Denial::ForeignOrStaleBasis)?;
                transaction
                    .push_batch(
                        WorkerIntentBatch::new("inbound-external-effect-completion").push(intent),
                     worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                    .map_err(|denial| match denial {
                        worth_relational::facade::mvcc::RelationalTransactionStagingDenial::AllocationDenied(allocation) => Denial::StagingAllocationDenied {
                            kind: allocation.kind(),
                            requested_payload_bytes: allocation.requested_payload_bytes(),
                        },
                        worth_relational::facade::mvcc::RelationalTransactionStagingDenial::CardinalityOverflow => Denial::StagingCardinalityOverflow,
                        worth_relational::facade::mvcc::RelationalTransactionStagingDenial::InputDirectoryAllocationDenied { requested_batches } => Denial::StagingInputDirectoryAllocationDenied { requested_batches },
                        worth_relational::facade::mvcc::RelationalTransactionStagingDenial::SavepointCapacityExhausted { .. }
                        | worth_relational::facade::mvcc::RelationalTransactionStagingDenial::SavepointIdentityExhausted
                        | worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationAuthorityRequired
                        | worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationModeMismatch => Denial::StagingUnavailable,
                    })?;
                let validated = transaction
                    .validate(runtime, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                    .map_err(|error| preparation_denial::validation_denial(&error))?;
                let mut candidate = runtime
                    .prepare_validated_proposal(validated)
                    .map_err(|error| preparation_denial::preparation_denial(&error))?;
                crate::domain_computation::primary_graph::index_maintenance_budget::prepare_candidate_with_cold_fallback(
                    runtime, &mut candidate, &self.graph.primary_index_ids, &before,
                    crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(),
                ).map_err(|_| Denial::IndexPreparationUnavailable)?;
                Ok(candidate)
            })();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
            result
        })
    }
}

fn string(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::from(value.into()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use preparation_denial::{
    preparation_denial as completion_preparation_denial,
    validation_denial as completion_validation_denial,
};
