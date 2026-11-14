//! Original accepted-output data from an exact owner-issued settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::{self as checkpoint, facts},
    application_output_demand::{
        WorthQueryAcceptedOutputCheckpointIdentity as Identity,
        WorthQueryAcceptedOutputCheckpointPosture as Posture,
    },
    WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
};
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity_for_settlement(
        &self,
        identity: &RecordedSettlementIdentity,
    ) -> Result<Option<Identity>, Denial> {
        let coordinate = identity.coordinate();
        let record = self
            .by_source
            .get(identity.source())
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
            .and_then(|generations| generations.get(&coordinate.generation))
            .and_then(|rows| rows.get(identity.slot()))
            .and_then(|cell| cell.get())
            .ok_or_else(|| unavailable("original output settlement row is unavailable"))?;
        if record.settlement_identity.as_ref() != identity {
            return Err(unavailable(
                "original output settlement differs from its selected row",
            ));
        }
        let Some(locator) = record.native_prior_checkpoint.as_ref() else {
            // Ordinary source/staged writers share the publication group but
            // are not accepted producer outputs.
            return if record.producing_implementation.is_none() {
                Ok(None)
            } else {
                Err(unavailable(
                    "producer output has no original checkpoint locator",
                ))
            };
        };
        let mut retained = if let Some(known) = locator.recovered() {
            known.clone()
        } else {
            Identity {
                implementation: record.producing_implementation.ok_or_else(|| {
                    unavailable("producer implementation identity is unavailable")
                })?,
                producer: std::sync::Arc::new(locator.producer().to_owned()),
                posture: Posture::StableReused,
                source: locator.source(),
                scope: identity.source().scope,
                source_partition: record
                    .source_partition_identity
                    .ok_or_else(|| unavailable("producer source partition is unavailable"))?,
                producer_dependency: record.producer_dependency_identity,
                idempotency_key: record.idempotency_key_identity,
                roles: std::sync::Arc::new(record.correspondence.checkpoint_roles()),
                producer_facts: None,
                producer_fact_wire_version: 0,
                metadata: None,
            }
        };
        if retained.producer_facts.is_none() {
            let source = record.checkpoint_source_facts().ok_or_else(|| {
                unavailable("completed producer has no original dependency facts")
            })?;
            let bytes = match record.native_output_witness() {
                Some(witness) => checkpoint::capture::output_facts::encode(&source, witness),
                None => facts::encode(&source),
            }
            .ok_or_else(|| unavailable("original producer facts cannot be encoded"))?;
            retained.producer_facts = Some(std::sync::Arc::new(bytes));
            retained.producer_fact_wire_version = facts::WIRE_VERSION;
        }
        Ok(Some(retained))
    }
    pub(in crate::domain_computation::primary_graph) fn encode_checkpoint_prerequisite(
        &self,
        identity: &RecordedSettlementIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<u8>, Denial> {
        let retained = self
            .checkpoint_identity_for_settlement(identity)?
            .ok_or_else(|| unavailable("consumed checkpoint root has no producer locator"))?;
        checkpoint::encode::body::encode_accepted(&retained, execution.allocation_policy())
            .map_err(|_| unavailable("original accepted output cannot be encoded"))
    }
}
fn unavailable(detail: &'static str) -> Denial {
    Denial::new(Kind::RetainedBasisUnavailable, detail)
}
