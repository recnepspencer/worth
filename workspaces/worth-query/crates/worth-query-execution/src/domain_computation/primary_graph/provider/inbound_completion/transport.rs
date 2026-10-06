//! Provider candidate for a completed installed outbound transport attempt.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, mvcc::PreparedRelationalCommitCandidate,
    transactions::RecordRef,
};

use super::super::WorthQueryPrimaryGraphProvider;
use super::{hex, string, WorthQueryInboundCompletionPreparationDenial as Denial};
use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::primary_graph::{
    WorthQueryCommittedDispatchOutboxObservation, WorthQueryInstalledTransportCompletionBinding,
};

impl WorthQueryPrimaryGraphProvider {
    /// Prepare the same one terminal Relational mutation used by authenticated
    /// inbound completion. Transport evidence is identified as such in the
    /// canonical row; no verifier claim or signed message is fabricated.
    pub(in crate::domain_computation::primary_graph) fn prepare_installed_transport_completion_candidate(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        dispatch: &WorthQueryExternalEffectDispatch,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<PreparedRelationalCommitCandidate, Denial> {
        let record = owner.record();
        let contract = record.inbound().ok_or(Denial::InstalledBindingMismatch)?;
        if contract != binding.contract()
            || record.effect() != contract.effect()
            || record.protocol_identity() != contract.protocol().identity()
            || record.protocol_version() != contract.protocol().version()
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        if !dispatch.matches_committed_owner(binding.runtime(), owner) {
            return Err(Denial::DispatchOwnerMismatch);
        }
        let Some(observation) = dispatch.causal_ladder().observation() else {
            return Err(Denial::DispatchOwnerMismatch);
        };
        let RecordRef::Entity(original_entity) = owner.record_ref() else {
            return Err(Denial::OriginalOutboxNotAnEntity);
        };
        let layout = self.graph.layout.provider_inbound_completion();
        let correlation = hex(record.correlation().digest().bytes());
        let fields = BTreeMap::from([
            (layout.correlation.clone(), string(correlation.clone())),
            (
                layout.family.clone(),
                string(record.correlation_family().as_str()),
            ),
            (layout.operation.clone(), string(binding.operation())),
            (layout.audience.clone(), string(binding.audience())),
            (layout.source.clone(), string(contract.source_identity())),
            (layout.expires_at.clone(), AspectValue::UInt64(0)),
            (layout.key_epoch.clone(), AspectValue::UInt64(0)),
            (layout.message.clone(), string("")),
            (
                layout.protocol.clone(),
                string(record.protocol_identity().as_str()),
            ),
            (
                layout.version.clone(),
                AspectValue::UInt64(u64::from(record.protocol_version().get())),
            ),
            (layout.meaning_digest.clone(), string("")),
            (layout.payload.clone(), string(hex(record.payload()))),
            (
                layout.original_commit.clone(),
                AspectValue::UInt64(owner.commit_reference().commit_id.0),
            ),
            (
                layout.original_branch.clone(),
                string(&owner.commit_reference().branch_id.0),
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
                    owner
                        .committed_product_publication()
                        .product_incarnation()
                        .ordinal(),
                ),
            ),
            (
                layout.terminal.clone(),
                string("installed-transport-completed"),
            ),
            (
                layout.provenance_kind.clone(),
                string("installed-transport-completion"),
            ),
            (
                layout.transport_attempt.clone(),
                string(hex(dispatch.causal_ladder().attempt().identity().bytes())),
            ),
            (
                layout.transport_observation.clone(),
                string(hex(observation.identity().bytes())),
            ),
        ]);
        self.prepare_completion_fields(basis, *original_entity, &correlation, fields)
    }
}
