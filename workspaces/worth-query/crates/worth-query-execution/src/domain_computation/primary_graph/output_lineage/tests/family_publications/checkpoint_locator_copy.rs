//! Descriptive locator selection/copy controls; no producer freshness claim.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::merge_native_checkpoint_priors;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity as Identity;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture as Posture;
use crate::domain_computation::primary_graph::output_lineage::native_prior_checkpoint::NativePriorCheckpointLocator;
use worth_relational::facade::mvcc::RequiredOutputProducingImplementation as Implementation;

fn court() -> Court {
    let mut court = Court::new();
    // Checkpoint preservation retains A's original pair even under installed build B.
    court.lineage.bind_implementation_build(
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "33333333-3333-3333-3333-333333333333",
        )
        .unwrap(),
    );
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
    recorded.producing_implementation = Some(original_implementation());
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

fn original_implementation() -> Implementation {
    Implementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap()
}

fn expected(court: &Court) -> Identity {
    Identity {
        implementation: original_implementation(),
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
    for recovered in [false, true] {
        // Missing-head recovery has its own retention policy. These new recovered
        // twins keep the exact head coordinates and vary only the original pair.
        let drifts: &[usize] = if recovered {
            &[6, 7]
        } else {
            &[0, 1, 2, 3, 4, 5, 6, 7]
        };
        for &drift in drifts {
            let mut candidate = expected.clone();
            let mut binding = TypeId::of::<Initial>();
            match drift {
                0 => {}
                1 => binding = TypeId::of::<Preserve>(),
                2 => candidate.idempotency_key = [3; 32],
                3 => candidate.source = [12; 32],
                4 => candidate.producer = Arc::new("other-producer".to_owned()),
                5 => Arc::make_mut(&mut candidate.roles)[1].entity = entity(4),
                6 => {
                    candidate.implementation =
                        Implementation::from_checkpoint([0x33; 16], [0x22; 32]).unwrap()
                }
                7 => {
                    candidate.implementation =
                        Implementation::from_checkpoint([0x11; 16], [0x44; 32]).unwrap()
                }
                _ => unreachable!(),
            }
            let mut rows = if drift == 0 {
                Vec::new()
            } else {
                vec![(candidate, None, Some(binding))]
            };
            let recovered_rows = if recovered {
                rows.drain(..)
                    .map(|(identity, _, binding)| (identity, binding))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let merged = merge_native_checkpoint_priors(
                &court.lineage,
                rows,
                recovered_rows,
                select(&court),
            )
            .unwrap();
            assert_eq!(merged.len(), 1);
            assert_eq!(
                merged[0].0, expected,
                "recovered={recovered}, drift={drift}"
            );
            assert!(merged[0].1.is_none());
        }
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

    let mut missing = self::court();
    let cell = missing
        .lineage
        .by_source
        .get_mut(&missing.source)
        .unwrap()
        .get_mut(&missing.coordinate.occurrence)
        .unwrap()
        .get_mut(&9)
        .unwrap();
    Arc::get_mut(&mut cell[0])
        .unwrap()
        .get_mut()
        .unwrap()
        .producing_implementation = None;
    let priors = select(&missing);
    use super::super::super::family_selection::{
        CheckpointPriorSelectionDenial, CheckpointPriorStructure,
    };
    assert!(matches!(
        priors[0].materialize_identity(),
        Err(CheckpointPriorSelectionDenial::Structural(
            CheckpointPriorStructure::MissingProducingImplementation
        ))
    ));
}

#[test]
fn missing_locator_still_checks_the_recorded_pair_before_retaining_accepted_data() {
    for recovered in [false, true] {
        // Complete matching pair, wrong build, wrong edition, and missing pair.
        for drift in 0..4 {
            let mut court = court();
            let expected = expected(&court);
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
            recorded.native_prior_checkpoint = None;
            if drift == 3 {
                recorded.producing_implementation = None;
            }
            let mut candidate = expected.clone();
            match drift {
                1 => {
                    candidate.implementation =
                        Implementation::from_checkpoint([0x33; 16], [0x22; 32]).unwrap()
                }
                2 => {
                    candidate.implementation =
                        Implementation::from_checkpoint([0x11; 16], [0x44; 32]).unwrap()
                }
                _ => {}
            }
            let priors = select(&court);
            assert!(priors[0].identity.is_none());
            assert!(priors[0].materialize_identity().unwrap().is_none());
            let (current, recovered_rows) = if recovered {
                (Vec::new(), vec![(candidate, Some(TypeId::of::<Initial>()))])
            } else {
                (
                    vec![(candidate, None, Some(TypeId::of::<Initial>()))],
                    Vec::new(),
                )
            };
            let merged =
                merge_native_checkpoint_priors(&court.lineage, current, recovered_rows, priors)
                    .unwrap();
            if drift == 0 {
                assert_eq!(merged.len(), 1);
                assert_eq!(merged[0].0, expected);
            } else {
                assert!(merged.is_empty(), "recovered={recovered}, drift={drift}");
            }
        }
    }
}
