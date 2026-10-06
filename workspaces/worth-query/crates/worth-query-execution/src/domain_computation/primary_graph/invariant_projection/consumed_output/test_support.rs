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
            identity,
            source_facts,
            upstream: Arc::from(upstream),
            verification_requirement: requirement,
            native_output_witness: None,
            selected_native_root: selected,
            _capacity: capacity,
            backing_capacity: None,
        }
    }
}
