use std::sync::Arc;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

impl ConsumedOutputEvidence {
    /// Test journeys retain the original identity, facts and selected native root.
    pub(in crate::domain_computation::primary_graph) fn retained_for_test(
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: impl Into<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >,
        upstream: Vec<Self>,
        requirement: Option<FullVerificationReason>,
        selected: Arc<PositionedRelationalSnapshot>,
    ) -> Self {
        let source_facts = source_facts.into();

        Self {
            _computation: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).certify_current().unwrap(),            identity,
            source_facts: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).retain_facts(source_facts).for_comparison().unwrap(),
            upstream: Arc::from(upstream),
            verification_requirement: requirement,
            native_output_witness: None,
            selected_native_root: selected,


        }
    }
}

impl ConsumedOutputEvidence {
    /// The same retained consumption carrier, with an actual sealed Native
    /// witness supplied by the restored-root reader's full comparison.
    pub(in crate::domain_computation::primary_graph) fn restored_with_witness_for_test(
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: impl Into<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >,
        selected: Arc<PositionedRelationalSnapshot>,
        witness: Arc<
            std::sync::OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >,
    ) -> Self {
        let mut retained = Self::retained_for_test(
            identity,
            source_facts,
            Vec::new(),
            Some(FullVerificationReason::CheckpointRestore),
            selected,
        );
        retained.native_output_witness = Some(witness);
        retained
    }
}
