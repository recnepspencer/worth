//! Descriptive family-selection controls, not producer freshness or R3 proof.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::{
    family_selection::{
        CheckpointPriorSelectionDenial as Denial, CheckpointPriorStructure,
        NativePriorCheckpointOutput,
    },
    invalidation::InvalidationEditAdmission,
};
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

fn select(
    court: &Court,
    work: u64,
    bytes: u64,
) -> Result<Vec<NativePriorCheckpointOutput>, Denial> {
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: bytes,
    });
    court.lineage.checkpoint_prior_outputs(
        court.source.runtime_authority,
        &court.source.schema,
        court.coordinate.occurrence,
        court.coordinate.generation,
        &mut admission,
    )
}

#[test]
fn checkpoint_selection_preserves_work_and_scratch_refusal_at_capture_boundary() {
    let court = Court::new();
    let denial = select(&court, 0, u64::MAX)
        .err()
        .expect("selection must refuse");
    assert!(matches!(
        &denial,
        Denial::Admission {
            stop: CompanionPreflightStop::WorkExhausted {
                required: 1,
                maximum: 0
            },
            ..
        }
    ));
    let captured = denial.into_capture_denial();
    assert!(captured.detail.contains("head inventory visit"));
    assert!(captured
        .detail
        .contains("WorkExhausted { required: 1, maximum: 0 }"));

    let denial = select(&court, u64::MAX, 0)
        .err()
        .expect("selection must refuse");
    assert!(
        matches!(&denial, Denial::Admission { stop: CompanionPreflightStop::PreparationMemoryExhausted { required, maximum: 0 }, .. } if *required > 0)
    );
    assert!(denial
        .into_capture_denial()
        .detail
        .contains("installed binding scratch"));
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
    let denial = select(&court, u64::MAX, u64::MAX)
        .err()
        .expect("selection must refuse");
    assert!(matches!(
        &denial,
        Denial::Structural(CheckpointPriorStructure::AmbiguousPublicationHeads)
    ));
    assert!(denial
        .into_capture_denial()
        .detail
        .contains("AmbiguousPublicationHeads"));
}
