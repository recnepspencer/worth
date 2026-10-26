//! Descriptive family-selection controls, not producer freshness or R3 proof.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::family_selection::CheckpointPriorSelectionDenial as Denial;
use crate::domain_computation::primary_graph::output_lineage::family_selection::CheckpointPriorStructure;
use crate::domain_computation::primary_graph::output_lineage::family_selection::NativePriorCheckpointOutput;

fn select(court: &Court) -> Result<Vec<NativePriorCheckpointOutput<'_>>, Denial> {
    court.lineage.checkpoint_prior_outputs(
        &worth_execution::ExecutionRequest::serial(&court.request),
        court.source.runtime_authority,
        &court.source.schema,
        court.coordinate.occurrence,
        court.coordinate.generation,
    )
}

#[test]
fn checkpoint_selection_reports_actual_ambiguous_head_without_relaxing_selection() {
    let mut court = Court::new();
    court.publish::<Initial>(
        9,
        Some(1),
        20,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        9,
        Some(1),
        20,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    let denial = select(&court).err().expect("selection must refuse");
    assert!(matches!(
        &denial,
        Denial::Structural(CheckpointPriorStructure::AmbiguousPublicationHeads)
    ));
    assert!(denial
        .into_capture_denial()
        .detail
        .contains("AmbiguousPublicationHeads"));
}
