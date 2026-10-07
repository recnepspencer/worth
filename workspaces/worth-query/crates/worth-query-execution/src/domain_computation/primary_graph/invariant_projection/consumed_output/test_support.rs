use std::sync::Arc;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    output_lineage::{invalidation::FullVerificationReason, RecordedSettlementIdentity},
    SourceInvalidationOwner,
};

impl ConsumedOutputEvidence {
    /// Test journeys use the same installed retention owner as real reads.
    pub(in crate::domain_computation::primary_graph) fn retained_for_test(
        owner: &SourceInvalidationOwner,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: Arc<[WorthQueryApplicationObservedFact]>,
        mut upstream: Vec<Self>,
        requirement: Option<FullVerificationReason>,
        selected: Arc<PositionedRelationalSnapshot>,
    ) -> Self {
        let mut admission = owner.edit_admission();
        Self::admit_backing(&mut upstream, owner, &mut admission)
            .expect("test carrier backing has installed capacity");
        let capacity = owner
            .retain_consumed_output(
                &source_facts,
                &selected,
                Self::metadata_bytes(),
                &mut admission,
            )
            .expect("test carrier has installed capacity");
        Self {
            _computation: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).certify_current().unwrap(),            identity,
            source_facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, source_facts).for_comparison().unwrap(),
            upstream: Arc::from(upstream),
            verification_requirement: requirement,
            native_output_witness: None,
            selected_native_root: selected,
            _capacity: capacity,
            backing_capacity: None,
        }
    }
}

impl ConsumedOutputEvidence {
    /// The same retained consumption carrier, with an actual sealed Native
    /// witness supplied by the restored-root reader's full comparison.
    pub(in crate::domain_computation::primary_graph) fn restored_with_witness_for_test(
        owner: &SourceInvalidationOwner,
        identity: Arc<RecordedSettlementIdentity>,
        source_facts: Arc<[WorthQueryApplicationObservedFact]>,
        selected: Arc<PositionedRelationalSnapshot>,
        witness: Arc<
            std::sync::OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >,
    ) -> Self {
        let mut retained = Self::retained_for_test(
            owner,
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
