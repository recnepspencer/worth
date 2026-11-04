//! Descriptive family-selection controls, not producer freshness or R3 proof.
use crate::domain_computation::primary_graph::output_lineage::tests::family_publications::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::denial::CheckpointPriorSelectionDenial as Denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::denial::CheckpointPriorStructure;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::NativePriorCheckpointOutput;

pub fn select(court: &Court) -> Result<Vec<NativePriorCheckpointOutput<'_>>, Denial> {
    court.lineage.checkpoint_prior_outputs_for_test(
        &worth_execution::ExecutionRequest::serial(&court.request),
        (*court.source.runtime_authority_for_test()),
        &(*court.source.schema_for_test()),
        (*court.coordinate.occurrence_for_test()),
        (*court.coordinate.generation_for_test()),
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
        .into_capture_denial_for_test()
        .detail
        .contains("AmbiguousPublicationHeads"));
}
