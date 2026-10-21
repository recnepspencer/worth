//! Descriptive locator selection/copy controls; no producer freshness claim.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::merge_native_checkpoint_priors;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity as Identity;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture as Posture;
use crate::domain_computation::primary_graph::output_lineage::native_prior_checkpoint::NativePriorCheckpointLocator;

fn court() -> Court {
    let mut court = Court::new();
    court.publish::<Initial>(
        9,
        Some(7),
        2,
        WorthQueryApplicationOutputPosture::Create,
        false,
    );
    let cell = court
        .lineage
        .by_source
        .get_mut(&court.source)
        .unwrap()
        .get_mut(&court.coordinate.occurrence)
        .unwrap()
        .get_mut(&9)
        .unwrap();
    let recorded = Arc::get_mut(&mut cell[0]).unwrap().get_mut().unwrap();
    recorded.native_prior_checkpoint = Some(NativePriorCheckpointLocator::Performed {
        producer: "fixture-producer".into(),
        source: [11; 32],
    });
    recorded.correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            TypeId::of::<Initial>(),
            TypeId::of::<()>(),
            std::iter::empty(),
            &expected_roles(),
            |_| Some(TypeId::of::<()>()),
            &worth_execution::ExecutionRequest::serial(
                &worth_execution::SerialRequest::from_memory(
                    worth_execution::SerialMemoryBudget::new(None),
                    worth_execution::CancellationToken::new(),
                    None,
                ),
            ),
        )
        .unwrap(),
    );
    court
}

fn expected_roles() -> Vec<WorthQueryCheckpointOutputRole> {
    vec![
        WorthQueryCheckpointOutputRole {
            role: "auxiliary".into(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "auxiliary-entity".into(),
            entity: entity(3),
        },
        WorthQueryCheckpointOutputRole {
            role: "output".into(),
            posture: WorthQueryApplicationOutputPosture::Create,
            entity_name: "fixture".into(),
            entity: entity(2),
        },
    ]
}

fn expected(court: &Court) -> Identity {
    Identity {
        producer: "fixture-producer".to_owned().into(),
        posture: Posture::Performed,
        source: [11; 32],
        scope: court.source.scope,
        source_partition: [7; 32],
        producer_dependency: None,
        idempotency_key: [2; 32],

        roles: expected_roles().into(),
        producer_facts: None,
        producer_fact_wire_version: 0,
        metadata: None,
    }
}

fn select<'a>(
    court: &'a Court,
) -> Vec<super::super::super::family_selection::NativePriorCheckpointOutput<'a>> {
    court
        .lineage
        .checkpoint_prior_outputs(
            &worth_execution::ExecutionRequest::serial(&court.request),
            court.source.runtime_authority,
            &court.source.schema,
            court.coordinate.occurrence,
            court.coordinate.generation,
        )
        .unwrap()
}

#[test]
fn accepted_native_head_skips_only_the_discarded_full_locator_copy() {
    let court = court();

    let priors = select(&court);

    let mut accepted = expected(&court);
    accepted.producer_facts = Some(Arc::new(vec![7, 8, 9]));
    accepted.producer_fact_wire_version = 4;
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(accepted.clone(), None, Some(TypeId::of::<Initial>()))],
        std::iter::empty(),
        priors,
    )
    .unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].0, accepted);
    assert!(merged[0].1.is_none());

    let mut rejected = accepted;
    rejected.idempotency_key = [3; 32];

    let priors = select(&court);
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(rejected, None, Some(TypeId::of::<Initial>()))],
        std::iter::empty(),
        priors,
    )
    .unwrap();
    assert_eq!(merged[0].0, expected(&court));
}

#[test]
fn absent_or_rejected_accepted_rows_preserve_the_complete_native_locator() {
    let court = court();
    let expected = expected(&court);
    for drift in 0..6 {
        let mut candidate = expected.clone();
        let mut binding = TypeId::of::<Initial>();
        match drift {
            0 => {}
            1 => binding = TypeId::of::<Preserve>(),
            2 => candidate.idempotency_key = [3; 32],
            3 => candidate.source = [12; 32],
            4 => candidate.producer = Arc::new("other-producer".to_owned()),
            5 => Arc::make_mut(&mut candidate.roles)[1].entity = entity(4),
            _ => unreachable!(),
        }
        let rows = if drift == 0 {
            Vec::new()
        } else {
            vec![(candidate, None, Some(binding))]
        };

        let priors = select(&court);

        let merged =
            merge_native_checkpoint_priors(&court.lineage, rows, std::iter::empty(), priors)
                .unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0, expected);
        assert!(merged[0].1.is_none());
    }
}

#[test]
fn required_native_locator_materializes_the_complete_identity() {
    let court = court();
    let priors = select(&court);
    assert_eq!(
        priors[0].materialize_identity().unwrap().unwrap(),
        expected(&court)
    );
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        Vec::new(),
        std::iter::empty(),
        select(&court),
    )
    .unwrap();
    assert_eq!(merged[0].0, expected(&court));
}
