//! D1178 read evidence: original revisions cross only a committed graph selection.
use super::read_evidence_fixture::{edit, note, other, SelectedWorld};
use super::*;
use worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial as ReadDenial;
use worth_testkit_relational::aspect_storage::aspect_key;

#[test]
fn a_committed_selection_bridges_the_original_field_and_aspect_revisions_of_its_member() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let expected = world.revision(execution, &world.selected, world.model, &note());
    let restored = world.revision(execution, &world.restored, world.model, &note());
    // Native gave the restored field a fresh revision: a direct comparison of
    // the original observation rejects a healthy selected version.
    assert_ne!(restored, expected);
    let edited = world.revision(execution, &world.edited, world.model, &note());
    let aspect = world.aspect(execution, &world.selected, world.model);
    let evidence = world.evidence(execution).unwrap();
    assert_eq!(evidence.model(), world.model);
    assert_eq!(
        Some(evidence.selection_commit()),
        world.restored.observation().commit_id()
    );
    let current = world.current();
    assert!(evidence
        .matches_field_revision(execution, &current, world.model, &note(), expected)
        .unwrap());
    assert!(evidence
        .matches_aspect_revision(
            execution,
            &current,
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
    // The replaced edit's revision is not the selected original.
    assert!(!evidence
        .matches_field_revision(execution, &current, world.model, &note(), edited)
        .unwrap());
    // A view older than the restoration cannot borrow its baseline.
    assert!(!evidence
        .matches_field_revision(
            execution,
            &world.view(&world.edited),
            world.model,
            &note(),
            expected
        )
        .unwrap());
}

#[test]
fn a_later_relevant_edit_or_aba_breaks_the_bridge_and_an_unrelated_field_does_not() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let expected = world.revision(execution, &world.selected, world.model, &note());
    let aspect = world.aspect(execution, &world.selected, world.model);
    let evidence = world.evidence(execution).unwrap();
    let field =
        |evidence: &worth_relational::facade::runtime::RelationalHistoricalSelectionRead| {
            evidence
                .matches_field_revision(execution, &world.current(), world.model, &note(), expected)
                .unwrap()
        };
    let whole = || {
        evidence
            .matches_aspect_revision(
                execution,
                &world.current(),
                world.model,
                &aspect_key("note"),
                aspect,
            )
            .unwrap()
    };
    edit(
        execution,
        &world.runtime,
        world.model,
        "other",
        "changed later",
    );
    assert!(field(&evidence), "an unrelated field edit keeps the proof");
    assert!(whole(), "another aspect's edit keeps this aspect's proof");
    edit(
        execution,
        &world.runtime,
        world.model,
        "note",
        "changed later",
    );
    assert!(!field(&evidence));
    assert!(!whole());
    // Restoring the selected value by an ordinary edit is ABA, not selection.
    edit(execution, &world.runtime, world.model, "note", "first");
    assert!(!field(&evidence));
    assert!(!whole());
    assert!(!evidence
        .matches_field_revision(
            execution,
            &world.current(),
            world.model,
            &other(),
            world.revision(execution, &world.selected, world.model, &other()),
        )
        .unwrap());
}

#[test]
fn an_outside_entity_cannot_use_the_selection_bridge() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let evidence = world.evidence(execution).unwrap();
    // The outside entity was edited before selection and is still edited: its
    // current revision equals the restoration baseline, but it was never
    // restored, so its original observation is stale.
    let expected = world.revision(execution, &world.selected, world.outside, &note());
    assert_eq!(
        world.revision(execution, &world.restored, world.outside, &note()),
        world.revision(execution, &world.edited, world.outside, &note())
    );
    assert!(!evidence
        .matches_field_revision(
            execution,
            &world.current(),
            world.outside,
            &note(),
            expected
        )
        .unwrap());
    let aspect = world.aspect(execution, &world.selected, world.outside);
    assert!(!evidence
        .matches_aspect_revision(
            execution,
            &world.current(),
            world.outside,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
}

#[test]
fn only_the_graph_selection_of_that_exact_selected_basis_admits_evidence() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let read =
        |selected: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
         restored: &worth_relational::facade::branch::AdmittedRelationalBranchBasis| {
            world
                .runtime
                .read_truth()
                .read_historical_selection(
                    execution,
                    &selected.observation(),
                    &restored.observation(),
                )
                .err()
        };
    let commit = world.restored.observation().commit_id().unwrap();
    assert!(matches!(
        read(&world.edited, &world.restored),
        Some(ReadDenial::SelectedBasisMismatch { commit: c }) if c == commit
    ));
    // The edited step is an ordinary commit, not a graph selection.
    assert!(matches!(
        read(&world.selected, &world.edited),
        Some(ReadDenial::NotGraphSelection { commit: Some(_) })
    ));
    edit(
        execution,
        &world.runtime,
        world.model,
        "other",
        "ordinary later",
    );
    let later = transfer_fixture::basis(&world.runtime);
    assert!(matches!(
        read(&world.selected, &later),
        Some(ReadDenial::NotGraphSelection { commit: Some(_) })
    ));
    let foreign = super::read_evidence_fixture::field_runtime();
    assert!(matches!(
        foreign.read_truth().read_historical_selection(
            execution,
            &world.selected.observation(),
            &world.restored.observation()
        ),
        Err(ReadDenial::ForeignOwner)
    ));
    let evidence = world.evidence(execution).unwrap();
    let expected = world.revision(execution, &world.selected, world.model, &note());
    let foreign_view = foreign
        .read_truth()
        .project_observation(&transfer_fixture::basis(&foreign).observation())
        .unwrap();
    assert!(matches!(
        evidence.matches_field_revision(execution, &foreign_view, world.model, &note(), expected),
        Err(ReadDenial::ForeignOwner)
    ));
}

#[test]
fn bounded_requests_refuse_and_worker_only_requests_poll_without_accounting() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(setup);
    let expected = world.revision(setup, &world.selected, world.model, &note());
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(1)),
    );
    let bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    assert!(matches!(
        world.evidence(&ExecutionRequest::serial(&bounded)),
        Err(ReadDenial::Request(_))
    ));
    let bounded = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationToken::new(),
        None,
    );
    assert!(matches!(
        world.evidence(&ExecutionRequest::serial(&bounded)),
        Err(ReadDenial::Request(_))
    ));
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let worker = &ExecutionRequest::leased(&lease);
    assert!(!worker.has_accounting());
    let evidence = world.evidence(worker).unwrap();
    assert_eq!(
        evidence
            .source_presence(worker, world.model, KindId(1))
            .unwrap(),
        worth_relational::facade::runtime::RelationalHistoricalSourcePresence::LiveInDomain
    );
    let source_bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    assert!(matches!(
        evidence.source_presence(
            &ExecutionRequest::serial(&source_bounded),
            world.model,
            KindId(1)
        ),
        Err(ReadDenial::Request(_))
    ));
    assert!(evidence
        .matches_field_revision(worker, &world.current(), world.model, &note(), expected)
        .unwrap());
    let aspect = world.aspect(setup, &world.selected, world.model);
    assert!(evidence
        .matches_aspect_revision(
            worker,
            &world.current(),
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
    let hold = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&world.selected.observation())
        .unwrap();
    let source = world
        .runtime
        .read_truth()
        .positioned_snapshot(&hold)
        .unwrap();
    assert!(evidence
        .includes_source(worker, &world.runtime, &source)
        .unwrap());
    world.runtime.snapshots().release_snapshot(&hold).unwrap();
    assert!(!worker.has_accounting());
}

#[test]
fn historical_upstream_sources_require_selected_ancestry_and_the_original_runtime() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(request);
    let evidence = world.evidence(request).unwrap();
    let position = |basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis| {
        let hold = world
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let position = world
            .runtime
            .read_truth()
            .positioned_snapshot(&hold)
            .unwrap();
        world.runtime.snapshots().release_snapshot(&hold).unwrap();
        position
    };
    assert!(evidence
        .includes_source(request, &world.runtime, &position(&world.selected))
        .unwrap());
    assert!(!evidence
        .includes_source(request, &world.runtime, &position(&world.edited))
        .unwrap());
    assert!(!evidence
        .includes_source(request, &world.runtime, &position(&world.restored))
        .unwrap());
    let foreign = super::read_evidence_fixture::field_runtime();
    assert!(matches!(
        evidence.includes_source(request, &foreign, &position(&world.selected)),
        Err(ReadDenial::ForeignOwner)
    ));
    let foreign_basis = transfer_fixture::basis(&foreign);
    let foreign_hold = foreign
        .snapshots()
        .snapshot_for_observation(&foreign_basis.observation())
        .unwrap();
    let foreign_source = foreign
        .read_truth()
        .positioned_snapshot(&foreign_hold)
        .unwrap();
    assert!(matches!(
        evidence.includes_source(request, &world.runtime, &foreign_source),
        Err(ReadDenial::ForeignOwner)
    ));
    foreign.snapshots().release_snapshot(&foreign_hold).unwrap();
}
