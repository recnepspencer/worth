//! Pure ordering/absence regression; these tokens assert no workflow authority.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::ObservedOperationOutput;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputRoleDescription, ApplicationMutationOutputPosture as Posture,
    ApplicationMutationOutputRoleCardinality as Cardinality,
};
use worth_relational::facade::identity::PartitionId;

fn outputs(slot: u64) -> ObservedOperationOutputs {
    ObservedOperationOutputs {
        binding: "certify".into(),
        entries: BTreeMap::from([(
            "reviewed".into(),
            ObservedOperationOutput {
                entity: "Round".into(),
                posture: Posture::Preserve,
                target: EntityId::new(PartitionId::new(7), slot, 1),
            },
        )]),
    }
}
fn occurrence(
    ordinal: u64,
    outputs: Option<&ObservedOperationOutputs>,
) -> PerformedProducerOccurrence<'_> {
    PerformedProducerOccurrence {
        occurrence: ordinal,
        binding: "certify",
        outputs,
    }
}
#[test]
fn latest_named_occurrence_wins_and_missing_latest_context_or_role_never_falls_back() {
    let matching = ApplicationOperationBodySuccessionOutputMatch::from_untrusted(
        "selected".into(),
        "certify".into(),
        ApplicationMutationOutputRoleDescription {
            name: "reviewed".into(),
            entity: "Round".into(),
            posture: Posture::Preserve,
            cardinality: Cardinality::ExactlyOne,
        },
    );
    let old = outputs(1);
    let new = outputs(2);
    for reverse in [false, true] {
        let records = if reverse {
            [occurrence(9, Some(&new)), occurrence(3, Some(&old))]
        } else {
            [occurrence(3, Some(&old)), occurrence(9, Some(&new))]
        };
        assert_eq!(
            latest_output(records.into_iter(), &matching).unwrap(),
            EntityId::new(PartitionId::new(7), 2, 1)
        );
    }
    let missing_role = ObservedOperationOutputs {
        binding: "certify".into(),
        entries: BTreeMap::new(),
    };
    for absent in [None, Some(&missing_role)] {
        let failure = latest_output(
            [occurrence(3, Some(&old)), occurrence(9, absent)].into_iter(),
            &matching,
        )
        .unwrap_err();
        assert_eq!(
            failure.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowPredecessorOutputUnavailable
        );
    }
}
