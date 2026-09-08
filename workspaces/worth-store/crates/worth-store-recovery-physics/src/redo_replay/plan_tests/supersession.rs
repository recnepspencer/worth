use super::*;

#[path = "supersession/fixture.rs"]
mod fixture;
use fixture::image;

#[test]
fn exact_wal_image_chain_supersedes_only_its_historical_claims() {
    let (first, _) = image(1, 1, 10, 1, 1, false);
    let (second, current) = image(2, 2, 11, 2, 1, false);
    let plan = plan_physical_redo(vec![first, second], vec![current], 64).unwrap();
    assert_eq!(plan.decisions().len(), 2);
    assert!(plan.decisions().iter().all(|decision| {
        decision.kind() == PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    }));
}

#[test]
fn supersession_scratch_admits_exact_budget_and_rejects_one_byte_less() {
    let (first, _) = image(1, 1, 10, 1, 1, false);
    let (second, current) = image(2, 2, 11, 2, 1, false);
    // Two target claims + two frames + three placements, across two members.
    let expected_scratch = 7 * 4_096;
    let limits = PhysicalRedoAdmissionLimits {
        recovery_memory_bytes: expected_scratch,
        targets: 64,
        distinct_targets: 64,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 64,
            record_identities: 64,
            placements: 64,
            segment_updates: 64,
            manifests: 64,
            total_entries: 192,
            inline_allocations: 64,
        },
    };
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let admitted = admit_physical_redo_members(
        vec![first.clone(), second.clone()],
        test_store(),
        format,
        limits,
    )
    .unwrap();
    assert_eq!(
        admitted
            .plan(vec![current])
            .unwrap()
            .supersession_scratch_bytes(),
        expected_scratch,
    );
    assert_eq!(
        admit_physical_redo_members(
            vec![first, second],
            test_store(),
            format,
            PhysicalRedoAdmissionLimits {
                recovery_memory_bytes: expected_scratch - 1,
                ..limits
            },
        )
        .err(),
        Some(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: expected_scratch,
            admitted: expected_scratch - 1,
        }),
    );
}

#[test]
fn newer_generation_without_exact_admitted_image_anchor_never_skips() {
    let (first, _) = image(1, 1, 10, 1, 1, false);
    let (second, current) = image(2, 2, 11, 2, 1, false);
    for forged in [
        observation(3, 12, current.frame_digest()),
        observation(2, 12, current.frame_digest()),
        observation(2, 11, [0xFF; 32]),
        RecoveryPageObservation::materialized(
            current.target(),
            current.page_lsn(),
            current.frame_digest(),
            worth_store_physical_format::RecordFrameCoordinate::new(
                RecordArtifactFile::Segment {
                    segment: 1,
                    generation: 9,
                },
                0,
                frame_len(),
            )
            .unwrap(),
            [3; 32],
        ),
    ] {
        assert_eq!(
            plan_physical_redo(vec![first.clone(), second.clone()], vec![forged], 64),
            Err(PhysicalRedoPlanningDenial::GenerationMismatch),
        );
    }
}

#[test]
fn missing_page_generation_or_discontinuous_root_lineage_never_supersedes() {
    for (page_generation, source_root) in [(3, 2), (2, 1), (2, 3)] {
        let (first, _) = image(1, 1, 10, 1, 1, false);
        let (second, current) = image(page_generation, source_root, 11, 2, 1, false);
        assert_eq!(
            plan_physical_redo(vec![first, second], vec![current], 64),
            Err(PhysicalRedoPlanningDenial::GenerationMismatch),
        );
    }
}

#[test]
fn later_self_consistent_frame_cannot_change_a_preserved_record() {
    let (first, _) = image(1, 1, 10, 1, 1, false);
    let (second, current) = image(2, 2, 11, 2, 1, true);
    assert_eq!(
        plan_physical_redo(vec![first, second], vec![current], 64),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
}

#[test]
fn impossible_segment_artifact_generation_transition_never_supersedes() {
    for artifact_generation in [1, 3] {
        let (first, _) = image(1, 1, 10, 1, 1, false);
        let (second, current) =
            fixture::image_generations((2, artifact_generation, 2), 11, 2, 1, false);
        assert_eq!(
            plan_physical_redo(vec![first, second], vec![current], 64).err(),
            Some(PhysicalRedoPlanningDenial::GenerationMismatch),
            "one root publication cannot reuse or skip the segment artifact generation"
        );
    }
}

#[test]
fn multiple_records_sharing_one_final_target_remain_exact_predecessors() {
    let (first, _) = image(1, 1, 10, 2, 2, false);
    let (second, current) = image(2, 2, 12, 3, 1, false);
    let plan = plan_physical_redo(vec![first, second], vec![current], 64).unwrap();
    assert_eq!(plan.decisions().len(), 3);
    assert!(plan.decisions().iter().all(|decision| {
        decision.kind() == PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    }));
}

#[test]
fn selected_intermediate_image_skips_predecessors_but_applies_its_successor() {
    let (first, first_image) = image(1, 1, 10, 1, 1, false);
    let (second, selected) = image(2, 2, 11, 2, 1, false);
    let (third, third_image) = image(3, 3, 12, 3, 1, false);
    let plan = plan_physical_redo(vec![first, second, third], vec![selected], 64).unwrap();
    let decisions = plan.resolved_decisions().collect::<Vec<_>>();
    assert_eq!(
        decisions
            .iter()
            .map(|decision| (
                decision.kind(),
                decision.record().lsn().get(),
                decision.target().identity(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
                10,
                first_image.target()
            ),
            (
                PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
                11,
                selected.target()
            ),
            (PhysicalRedoDecisionKind::Apply, 12, third_image.target()),
        ],
    );
    assert_eq!(
        decisions[2].prior(),
        PhysicalRedoDecisionPrior::Page(selected)
    );
}

#[test]
fn incomplete_observed_lsn_cannot_anchor_a_historical_skip() {
    let (first, first_image) = image(1, 1, 10, 1, 1, false);
    let (second, complete) = image(2, 2, 11, 3, 2, false);
    let control =
        plan_physical_redo(vec![first.clone(), second.clone()], vec![complete], 64).unwrap();
    assert_eq!(control.decisions().len(), 3);
    assert!(control.decisions().iter().all(|decision| {
        decision.kind() == PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    }));
    let RecoveryPageSource::Materialized {
        coordinate,
        routing_identity,
    } = complete.source()
    else {
        panic!("the canonical fixture supplies a materialized image")
    };
    assert_eq!(complete.page_lsn(), 12);
    // Mutate only the observed fact to isolate the planner's LSN requirement.
    // This is not a claim that intact-page acquisition can produce inconsistent facts.
    let incomplete = RecoveryPageObservation::materialized(
        complete.target(),
        11,
        complete.frame_digest(),
        coordinate,
        routing_identity,
    );
    let admitted = fixture::admitted_images(vec![first.clone(), second.clone()]);
    let complete_predecessors =
        super::super::supersession::observed_predecessors(&admitted.members, &[complete]).unwrap();
    assert_eq!(
        complete_predecessors,
        BTreeSet::from([(10, first_image.target())])
    );
    // A later same-generation Apply also rejects, so inspect the admitted
    // prepass to prove that incomplete progress grants no historical skip.
    assert!(
        super::super::supersession::observed_predecessors(&admitted.members, &[incomplete],)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        plan_physical_redo(vec![first, second], vec![incomplete], 64).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch),
    );
}

#[test]
fn separate_operations_cannot_publish_the_same_page_generation() {
    let (first, first_image) = image(1, 1, 10, 1, 1, false);
    let (second, second_image) = image(1, 2, 11, 2, 1, false);
    assert_ne!(first.operation(), second.operation());
    assert_ne!(
        first.group().group_identity(),
        second.group().group_identity()
    );
    for (member, observed) in [(first.clone(), first_image), (second.clone(), second_image)] {
        let control = plan_physical_redo(vec![member], vec![observed], 64).unwrap();
        assert_eq!(control.decisions().len(), 1);
        assert_eq!(
            control.decisions()[0].kind(),
            PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
        );
    }
    // Both members admit individually. Their continuous WAL conflicts before
    // page observation; removing the cross-operation guard must not go green
    // through an incidental missing-observation failure instead.
    assert_eq!(
        plan_physical_redo(vec![first, second], Vec::new(), 64).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch),
    );
}
