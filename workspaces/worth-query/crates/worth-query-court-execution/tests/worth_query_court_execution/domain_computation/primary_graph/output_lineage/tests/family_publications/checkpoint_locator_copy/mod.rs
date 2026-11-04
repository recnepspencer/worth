//! Descriptive locator selection/copy controls; no producer freshness claim.
use crate::domain_computation::primary_graph::output_lineage::tests::family_publications::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::capture::native_priors::merge as merge_native_checkpoint_priors;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity as Identity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture as Posture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_prior_checkpoint::NativePriorCheckpointLocator;
use worth_relational::facade::mvcc::RequiredOutputProducingImplementation as Implementation;

pub fn court() -> Court {
    let mut court = Court::new();
    // Checkpoint preservation retains A's original pair even under installed build B.
    court.lineage.bind_implementation_build_for_test(
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
    let cell = (*court.lineage.by_source_mut_for_test())
        .get_mut(&court.source)
        .unwrap()
        .get_mut(&(*court.coordinate.occurrence_for_test()))
        .unwrap()
        .get_mut(&9)
        .unwrap();
    let recorded = Arc::get_mut(&mut cell[0]).unwrap().get_mut().unwrap();
    (*recorded.producing_implementation_mut_for_test()) = Some(original_implementation());
    (*recorded.native_prior_checkpoint_mut_for_test()) =
        Some(NativePriorCheckpointLocator::Performed {
            producer: "fixture-producer".into(),
            source: [11; 32],
        });
    (*recorded.correspondence_mut_for_test()) = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<Initial>(),
            TypeId::of_for_test::<()>(),
            std::iter::empty(),
            &expected_roles(),
            |_| Some(TypeId::of_for_test::<()>()),
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

pub fn expected_roles() -> Vec<WorthQueryCheckpointOutputRole> {
    vec![
        WorthQueryCheckpointOutputRole::new_for_test(
            "auxiliary".into(),
            WorthQueryApplicationOutputPosture::Preserve,
            "auxiliary-entity".into(),
            entity(3),
        ),
        WorthQueryCheckpointOutputRole::new_for_test(
            "output".into(),
            WorthQueryApplicationOutputPosture::Create,
            "fixture".into(),
            entity(2),
        ),
    ]
}

pub fn original_implementation() -> Implementation {
    Implementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap()
}

pub fn expected(court: &Court) -> Identity {
    Identity::new_for_test(
        original_implementation(),
        "fixture-producer".to_owned().into(),
        Posture::Performed,
        [11; 32],
        court.source.scope,
        [7; 32],
        None,
        [2; 32],
        expected_roles().into(),
        None,
        0,
        None,
    )
}

pub fn select<'a>(
    court: &'a Court,
) -> Vec<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::NativePriorCheckpointOutput<'a>>{
    court
        .lineage
        .checkpoint_prior_outputs_for_test(
            &worth_execution::ExecutionRequest::serial(&court.request),
            (*court.source.runtime_authority_for_test()),
            &(*court.source.schema_for_test()),
            (*court.coordinate.occurrence_for_test()),
            (*court.coordinate.generation_for_test()),
        )
        .unwrap()
}

#[test]
fn accepted_native_head_skips_only_the_discarded_full_locator_copy() {
    let court = court();

    let priors = select(&court);

    let mut accepted = expected(&court);
    (*accepted.producer_facts_mut_for_test()) = Some(Arc::new(vec![7, 8, 9]));
    (*accepted.producer_fact_wire_version_mut_for_test()) = 4;
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(
            accepted.clone(),
            None,
            Some(TypeId::of_for_test::<Initial>()),
        )],
        std::iter::empty(),
        priors,
    )
    .unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].0, accepted);
    assert!(merged[0].1.is_none());

    let mut rejected = accepted;
    (*rejected.idempotency_key_mut_for_test()) = [3; 32];

    let priors = select(&court);
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(rejected, None, Some(TypeId::of_for_test::<Initial>()))],
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
            let mut binding = TypeId::of_for_test::<Initial>();
            match drift {
                0 => {}
                1 => binding = TypeId::of_for_test::<Preserve>(),
                2 => (*candidate.idempotency_key_mut_for_test()) = [3; 32],
                3 => (*candidate.source_mut_for_test()) = [12; 32],
                4 => (*candidate.producer_mut_for_test()) = Arc::new("other-producer".to_owned()),
                5 => {
                    (*Arc::make_mut(&mut (*candidate.roles_mut_for_test()))[1]
                        .entity_mut_for_test()) = entity(4)
                }
                6 => {
                    (*candidate.implementation_mut_for_test()) =
                        Implementation::from_checkpoint([0x33; 16], [0x22; 32]).unwrap()
                }
                7 => {
                    (*candidate.implementation_mut_for_test()) =
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
        priors[0].materialize_identity_for_test().unwrap().unwrap(),
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

    let mut missing = crate::domain_computation::primary_graph::output_lineage::tests::family_publications::checkpoint_locator_copy::court();
    let cell = (*missing.lineage.by_source_mut_for_test())
        .get_mut(&missing.source)
        .unwrap()
        .get_mut(&(*missing.coordinate.occurrence_for_test()))
        .unwrap()
        .get_mut(&9)
        .unwrap();
    (*Arc::get_mut(&mut cell[0])
        .unwrap()
        .get_mut()
        .unwrap()
        .producing_implementation_mut_for_test()) = None;
    let priors = select(&missing);
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::denial::CheckpointPriorSelectionDenial;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::denial::CheckpointPriorStructure;
    assert!(matches!(
        priors[0].materialize_identity_for_test(),
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
            let cell = (*court.lineage.by_source_mut_for_test())
                .get_mut(&court.source)
                .unwrap()
                .get_mut(&(*court.coordinate.occurrence_for_test()))
                .unwrap()
                .get_mut(&9)
                .unwrap();
            let recorded = Arc::get_mut(&mut cell[0]).unwrap().get_mut().unwrap();
            (*recorded.native_prior_checkpoint_mut_for_test()) = None;
            if drift == 3 {
                (*recorded.producing_implementation_mut_for_test()) = None;
            }
            let mut candidate = expected.clone();
            match drift {
                1 => {
                    (*candidate.implementation_mut_for_test()) =
                        Implementation::from_checkpoint([0x33; 16], [0x22; 32]).unwrap()
                }
                2 => {
                    (*candidate.implementation_mut_for_test()) =
                        Implementation::from_checkpoint([0x11; 16], [0x44; 32]).unwrap()
                }
                _ => {}
            }
            let priors = select(&court);
            assert!((*priors[0].identity_for_test()).is_none());
            assert!(priors[0].materialize_identity_for_test().unwrap().is_none());
            let (current, recovered_rows) = if recovered {
                (
                    Vec::new(),
                    vec![(candidate, Some(TypeId::of_for_test::<Initial>()))],
                )
            } else {
                (
                    vec![(candidate, None, Some(TypeId::of_for_test::<Initial>()))],
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
