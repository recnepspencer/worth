//! Creation grammar is proved through real publication and public DATA admission.
use super::super::owner_evidence::journey::Journey;
use super::wire::*;
use super::*;
use worth_foundational::facade::WorldDurableJournal as Journal;
#[test]
fn selected_creation_descriptions_preserve_shared_commits_and_historical_references() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let first = match create(&f, &request, &parent, "a-historical-child", false, false) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("actual first child: {other:?}"),
    };
    let second = match create(&f, &request, &first, "z-grandchild", false, false) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("actual grandchild: {other:?}"),
    };
    assert_eq!(first.selected_commit(), parent.selected_commit());
    assert_eq!(second.selected_commit(), parent.selected_commit());
    let mut journey = Journey::from_fixture(f);
    journey.advance(&request, parent.clone(), 1);
    let f = journey.fixture;
    let current = f
        .world
        .observation_port()
        .observe_product_branch(parent.branch_identity())
        .unwrap();
    assert_ne!(parent.selected_commit(), current.selected_commit());
    let third = match create(&f, &request, &current, "b-newer-child", false, false) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("actual newer child: {other:?}"),
    };
    assert_eq!(third.selected_commit(), current.selected_commit());
    let before = descriptions(&f, &request);
    assert_eq!(before.len(), 4);
    let checkpoint = f
        .world
        .checkpoint_port()
        .unwrap()
        .prepare_home_checkpoint(&request)
        .unwrap();
    checkpoint
        .validate_previous_selected(checkpoint.bytes(), &request)
        .unwrap();
    let data = PreparedWorldHomeData::from_home_source(
        f.recorder.binding().clone(),
        f.recorder
            .data_snapshot(Some(immutable(&request, checkpoint.bytes())), |_| {}),
        &request,
    )
    .unwrap();
    let catalog = data.native_selection().unwrap();
    use worth_relational::facade::durability::RelationalWorldSourceDescription as NativeSource;
    let native = f.native.owner_component_services().basis_port();
    let old = NativeSource::Recorded(
        native
            .export_recorded_retained_component(&request, parent.basis().relational_basis())
            .unwrap(),
    );
    let current_native = NativeSource::Recorded(
        native
            .export_recorded_retained_component(&request, current.basis().relational_basis())
            .unwrap(),
    );
    assert_ne!(old, current_native);
    assert_eq!(
        catalog.heads().len(),
        4,
        "one exact Native source per World branch"
    );
    assert_eq!(
        catalog
            .heads()
            .iter()
            .filter(|source| *source == &old)
            .count(),
        2
    );
    assert_eq!(
        catalog
            .heads()
            .iter()
            .filter(|source| *source == &current_native)
            .count(),
        2
    );
    assert_eq!(catalog.originals().len(), 1);
    drop((
        catalog, data, checkpoint, parent, first, second, third, current,
    ));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request), before);
    f.finish(&request);
}
fn immutable(
    request: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> worth_execution::ExecutionImmutableBytes {
    let mut out =
        worth_execution::ExecutionRequestArrayBuilder::allocate(*request, bytes.len()).unwrap();
    out.extend_from_slice(bytes).unwrap();
    out.seal_immutable_bytes().unwrap()
}
#[test]
fn creation_original_refuses_custody_already_reserved_by_an_operation() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 1);
    let f = journey.fixture;
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let child = create(&f, &request, &parent, "child", false, false);
    let denial = refusal(&f, &request, |rows| {
        let open = &rows.iter().find(|row| tag(&row.2) == 2).unwrap().2;
        let at = association_end(open) + 8;
        let custody = open[at..at + 8].to_vec();
        for row in rows.iter_mut().filter(|row| matches!(tag(&row.2), 7 | 8)) {
            let at = fields(&row.2).custody;
            row.2[at..at + 8].copy_from_slice(&custody);
            repair(&mut row.2);
        }
    });
    assert!(matches!(denial, WorldReadmissionDenial::DuplicateIdentity));
    drop((child, parent));
    f.finish(&request);
}
#[test]
fn selected_creation_refuses_a_missing_exact_source_reference() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let child = create(&f, &request, &parent, "child", false, false);
    let denial = refusal(&f, &request, |rows| {
        for row in rows.iter_mut().filter(|row| matches!(tag(&row.2), 7 | 8)) {
            let at = fields(&row.2).source_generation;
            row.2[at..at + 8].copy_from_slice(&7u64.to_le_bytes());
            repair(&mut row.2);
        }
    });
    assert!(matches!(denial, WorldReadmissionDenial::MissingPredecessor));
    drop((child, parent));
    f.finish(&request);
}
#[test]
fn selected_creation_refuses_a_release_of_the_same_original() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let child = create(&f, &request, &parent, "child", false, false);
    let denial = refusal(&f, &request, |rows| {
        let intent = &rows.iter().find(|row| tag(&row.2) == 7).unwrap().2;
        let release = release_from_intent(intent);
        let next = rows
            .iter()
            .filter(|row| row.0 == Journal::Custody)
            .map(|row| row.1)
            .max()
            .unwrap()
            + 1;
        rows.push((Journal::Custody, next, release));
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::ConflictingDisposition
    ));
    drop((child, parent));
    f.finish(&request);
}
#[test]
fn creation_codec_refuses_owner_evidence_for_an_exact_reuse() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let child = create(&f, &request, &parent, "child", false, false);
    let denial = refusal(&f, &request, |rows| {
        let row = rows.iter_mut().find(|row| tag(&row.2) == 8).unwrap();
        poison_reuse_owners(&mut row.2);
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::IdentityMismatch
        ))
    ));
    drop((child, parent));
    f.finish(&request);
}
#[test]
fn creation_artifact_refuses_the_retired_wire_version() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let child = create(&f, &request, &parent, "child", false, false);
    let denial = refusal(&f, &request, |rows| {
        let row = rows.iter_mut().find(|row| tag(&row.2) == 7).unwrap();
        row.2[8..12].copy_from_slice(&4u32.to_le_bytes());
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    drop((child, parent));
    f.finish(&request);
}
