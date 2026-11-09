//! Codec courts consume real public publication and protected DATA doors.
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_runtime_world::facade::*;
pub(super) mod journey;
use journey::*;
mod authored_retirement;
mod durable_cancellation;
mod transport;
mod unclean_signal;

#[test]
fn every_occurrence_owner_set_round_trips_without_fabricating_a_component() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    for tag in [1, 2, 3] {
        let journey = Journey::new(&request, tag);
        assert_eq!(journey.occurrence, journey.expected);
        let checkpoint = journey.checkpoint(&request);
        checkpoint
            .validate_previous_selected(checkpoint.bytes(), &request)
            .unwrap();
        let catalog = journey.data(&request).native_selection().unwrap();
        assert_eq!(catalog.originals().len(), usize::from(tag != 2));
        if let Some(signal) = &journey.signal {
            assert_eq!(
                checkpoint
                    .bytes()
                    .windows(signal.len())
                    .filter(|part| *part == signal)
                    .count(),
                1
            );
        }
        drop((checkpoint, catalog));
        journey.fixture.finish(&request);
    }
}
#[test]
fn occurrence_owner_set_must_match_its_original_open_intent() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 2);
    let offset = journey.open.len();
    let denial = journey.refusal(&request, None, |bytes| {
        bytes[offset] = 1;
        bytes.truncate(offset + 1);
        bytes.extend_from_slice(&[1u8; 48]);
        bytes.extend_from_slice(&1u64.to_le_bytes());
        let length = bytes.len() - 52;
        bytes[12..20].copy_from_slice(&(length as u64).to_le_bytes());
        rehash(bytes)
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::OccurrenceEvidenceMismatch
    ));
    journey.fixture.finish(&request);
}
#[test]
fn relational_only_world_artifact_version_refuses_after_the_owner_tag_cutover() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 1);
    let denial = journey.refusal(&request, None, |bytes| {
        bytes[8..12].copy_from_slice(&3u32.to_le_bytes())
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    journey.fixture.finish(&request);
}
#[test]
fn signal_occurrence_description_refuses_a_foreign_home_before_graph_admission() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 2);
    let foreign = foreign_signal_bytes(&request);
    let offset = journey.open.len() + 1;
    let denial = journey.refusal(&request, None, |bytes| {
        bytes.truncate(offset);
        bytes.extend_from_slice(&(foreign.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&foreign);
        bytes.resize(
            bytes.len()
                + worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
                - foreign.len(),
            0,
        );
        bytes.extend_from_slice(&1u64.to_le_bytes());
        let length = bytes.len() - 52;
        bytes[12..20].copy_from_slice(&(length as u64).to_le_bytes());
        rehash(bytes);
    });
    assert!(matches!(denial, WorldReadmissionDenial::ForeignOwner));
    journey.fixture.finish(&request);
}
#[test]
fn signal_only_occurrences_inherit_the_exact_native_source_without_a_native_original() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 2);
    let catalog = journey.data(&request).native_selection().unwrap();
    assert!(catalog.originals().is_empty());
    let (_, basis) = journey
        .fixture
        .native
        .observe_branch(&journey.fixture.native.main_branch_identity())
        .unwrap();
    let recorded = journey
        .fixture
        .native
        .owner_component_services()
        .basis_port()
        .export_recorded_current_component(&request, &basis)
        .unwrap();
    assert_eq!(
        catalog.heads(),
        &[
            worth_relational::facade::durability::RelationalWorldSourceDescription::Recorded(
                recorded
            )
        ]
    );
    drop((catalog, basis));
    journey.fixture.finish(&request);
}
#[test]
fn signal_only_successors_preserve_a_preceding_mixed_native_effect_source() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut journey = Journey::new(&request, 3);
    let before = journey.data(&request).native_selection().unwrap();
    let head = journey
        .fixture
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    journey.advance(&request, head, 2);
    let after = journey.data(&request).native_selection().unwrap();
    assert_eq!(before.originals().len(), 1);
    assert_eq!(after.originals().len(), 1);
    assert_eq!(after.heads(), before.heads());
    drop((before, after));
    journey.fixture.finish(&request);
}
#[test]
fn embedded_signal_envelope_version_is_checked_before_world_occurrence_admission() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 2);
    let denial = journey.refusal(&request, None, |bytes| {
        let offset = bytes
            .windows(8)
            .position(|part| part == b"WSIGOWNR")
            .unwrap();
        bytes[offset + 8..offset + 12].copy_from_slice(&2u32.to_le_bytes());
        rehash(bytes)
    });
    assert!(matches!(
        denial,
        WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    journey.fixture.finish(&request);
}

fn foreign_signal_bytes(request: &ExecutionRequest<'_, '_>) -> Vec<u8> {
    use worth_signal::facade::durability::{
        SignalDurableBranchDescriptor, SignalRecordedSourcePoint,
    };
    let owner = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([8; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(0),
    );
    let branch = DurableSignalBranchIdentity::from_recorded(owner, 1);
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, 1),
            1,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(owner, 1),
        SignalDefinitionTransitionReference::from_recorded(
            SignalDefinitionTransitionIdentity::from_recorded(owner, 1),
            [4; 32],
        ),
    )
    .unwrap();
    SignalRecordedSourcePoint::from_recorded(
        SignalBranchAdvanceReference::from_recorded(
            SignalBranchAdvanceIdentity::from_recorded(owner, 1),
            [5; 32],
        ),
        std::num::NonZeroU64::MIN,
        descriptor,
    )
    .unwrap()
    .encode_bytes(request)
    .unwrap()
    .as_ref()
    .to_vec()
}

#[test]
fn foreign_program_source_owner_refuses_before_an_open_is_published() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = super::artifact_source::Fixture::journal(&request);
    let root = match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("bootstrap: {other:?}"),
    };
    let before = fixture.recorder.heads();
    let revision = [3; 32];
    let foreign = worth_proof::InstalledBranchProgramSourceOwner::fresh().admit_source(
        revision,
        world_branch_program_source_commitment(&root, revision).unwrap(),
    );
    let outcome = fixture
        .world
        .publication_port()
        .open_publication_without_signal(
            &request,
            root.clone(),
            foreign,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            &RuntimeWorldCancellationSource::new().token(),
            None,
        );
    assert!(matches!(
        outcome,
        WorldOpenAttemptOutcome::Refused(WorldOpenAttemptRefusal::ProgramSource(
            worth_proof::BranchProgramSourceAdmissionDenial::ForeignOwner
        ))
    ));
    assert_eq!(fixture.recorder.heads(), before);
    drop(root);
    fixture.finish(&request);
}
#[test]
fn selected_program_source_refuses_a_different_head() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut journey = Journey::new(&request, 2);
    let before = journey
        .fixture
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let original = journey.fixture.recorder.program_source(&before);
    journey.advance(&request, before, 2);
    let head = journey
        .fixture
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let positions = journey.fixture.recorder.heads();
    let outcome = journey
        .fixture
        .world
        .publication_port()
        .open_publication_without_signal(
            &request,
            head.clone(),
            original,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            &RuntimeWorldCancellationSource::new().token(),
            None,
        );
    assert!(matches!(
        outcome,
        WorldOpenAttemptOutcome::Refused(WorldOpenAttemptRefusal::ProgramSource(
            worth_proof::BranchProgramSourceAdmissionDenial::SourceMismatch
        ))
    ));
    assert_eq!(journey.fixture.recorder.heads(), positions);
    drop(head);
    journey.fixture.finish(&request);
}
