//! Selected package eligibility is checked before Native owners are published.
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    CheckpointAdmissionAuthenticationMismatch as Authentication,
    CheckpointAdmissionBindingMismatch as Binding, RelationalCheckpointAdmissionRefusal as Refusal,
};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublication,
};

#[derive(Clone, Copy, Debug)]
enum Damage {
    CompletePredecessor,
    CheckpointPredecessor,
    FutureVersion,
    MissingImage,
    ChangedImage,
    ForeignHome,
}

#[test]
fn selected_home_admission_refuses_predecessors_missing_foreign_and_damaged_images() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let foreign = capture(&directory.path().join("foreign"), &execution);
    for (index, damage) in [
        Damage::CompletePredecessor,
        Damage::CheckpointPredecessor,
        Damage::FutureVersion,
        Damage::MissingImage,
        Damage::ChangedImage,
        Damage::ForeignHome,
    ]
    .into_iter()
    .enumerate()
    {
        let path = directory.path().join(index.to_string());
        let original = capture(&path, &execution);
        let home = crate::durable_run::open(&path, &execution).unwrap();
        assert_eq!(home.relational().cold_storage_reads().member_reads, 0);
        let records = home.physical_serving_for_test().anchored_records().unwrap();
        let name = AnchoredRecordName::new("home.checkpoint").unwrap();
        let selected = records.read(name, 1 << 30).unwrap().unwrap();
        assert_eq!(selected.bytes(), original);
        let members = selected
            .members()
            .iter()
            .map(|locator| AnchoredRecordMember::Carried { locator: *locator })
            .collect();
        records
            .publish(
                AnchoredRecordPublication::new(
                    name,
                    selected.version().generation(),
                    rewrite(original, &foreign, damage),
                    crate::open_refusals::home_placement(),
                    crate::open_refusals::deadline(),
                )
                .with_members(members),
            )
            .unwrap();
        drop(selected);
        // Simulate loss of the actual owner after its physical root decision;
        // orderly close would replace the value the refusal must inspect.
        home.abort_for_test();
        let refused = match crate::durable_run::open(&path, &execution) {
            Err(refused) => refused,
            Ok(_) => panic!("{damage:?} selected package must refuse"),
        };
        let Some(worth_store_application::ApplicationHomeOpenRefusal::Relational(
            worth_store_relational::RelationalSectionReadmissionDenial::AdmissionImage(cause),
        )) = refused.source_cause()
        else {
            panic!("{damage:?} must refuse at image eligibility: {refused:?}");
        };
        match damage {
            Damage::CompletePredecessor => {
                assert_predecessor(cause, "worth.store.relational.complete-section", 1);
            }
            Damage::CheckpointPredecessor => {
                assert_predecessor(cause, "worth.store.relational.checkpoint-section", 3);
            }
            Damage::FutureVersion => {
                let Refusal::UnsupportedVersion(version) = cause else {
                    panic!("typed future version: {cause:?}");
                };
                assert_eq!(version.produced().get(), 99);
                assert_eq!(version.posture(), worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow);
            }
            Damage::MissingImage => assert_eq!(cause, &Refusal::MissingArtifact),
            Damage::ChangedImage => assert_eq!(
                cause,
                &Refusal::AuthenticationMismatch(Authentication::Directory)
            ),
            Damage::ForeignHome => {
                assert_eq!(cause, &Refusal::BindingMismatch(Binding::Home));
            }
        }
    }
}

fn assert_predecessor(cause: &Refusal, name: &str, produced: u32) {
    let Refusal::PredecessorFormat { protocol, version } = cause else {
        panic!("typed predecessor: {cause:?}");
    };
    assert_eq!(protocol.as_str(), name);
    assert_eq!(version.produced().get(), produced);
    assert_eq!(
        version.posture(),
        worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture::Retired
    );
}

fn capture(path: &std::path::Path, request: &ExecutionRequest<'_, '_>) -> Vec<u8> {
    let fixture = super::coupled_home::workload(path, true, request);
    let home = fixture.home;
    let committed = crate::workload::commit(
        home.relational().runtime(),
        request,
        &worth_relational::facade::history::BranchId("main".into()),
        crate::batches::create(1),
    )
    .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    home.checkpoint(request).unwrap();
    assert!(matches!(
        home.close(request)
            .unwrap_or_else(|refused| panic!("World close: {:?}", refused.reason())),
        worth_store_application::ApplicationHomeCloseOutcome::Closed { .. }
    ));
    let home = crate::durable_run::open(path, request).unwrap();
    let selected = home
        .physical_serving_for_test()
        .anchored_records()
        .unwrap()
        .read(AnchoredRecordName::new("home.checkpoint").unwrap(), 1 << 30)
        .unwrap()
        .unwrap();
    let bytes = selected.into_bytes();
    assert_eq!(&bytes[34..42], b"WSRCAI01");
    home.abort_for_test();
    bytes
}

fn rewrite(mut frame: Vec<u8>, foreign: &[u8], damage: Damage) -> Vec<u8> {
    let integer =
        |bytes: &[u8], at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize;
    let end = 34 + integer(&frame, 16);
    let mut native = frame[34..end].to_vec();
    match damage {
        Damage::CompletePredecessor => {
            let source_length = integer(&native, 172);
            let mut old = b"WSRCSRC1".to_vec();
            old.extend_from_slice(&1u16.to_le_bytes());
            old.extend_from_slice(&native[12..68]);
            old.extend_from_slice(&native[292..292 + source_length]);
            native = old;
        }
        Damage::CheckpointPredecessor => {
            native = b"WSRCKPT1".to_vec();
            native.extend_from_slice(&3u16.to_le_bytes());
        }
        Damage::FutureVersion => native[8..12].copy_from_slice(&99u32.to_le_bytes()),
        Damage::MissingImage => {
            let start = 292 + integer(&native, 172);
            let finish = start + integer(&native, 180);
            native.drain(start..finish);
            native[180..188].copy_from_slice(&0u64.to_le_bytes());
        }
        Damage::ChangedImage => {
            let start = 292 + integer(&native, 172);
            native[start] ^= 1;
        }
        Damage::ForeignHome => {
            let foreign_end = 34 + integer(foreign, 16);
            native = foreign[34..foreign_end].to_vec();
            frame.truncate(34);
            frame[26..34].copy_from_slice(&foreign[26..34]);
            frame.extend_from_slice(&native);
            frame.extend_from_slice(&foreign[foreign_end..]);
            frame[16..24].copy_from_slice(&(native.len() as u64).to_le_bytes());
            return frame;
        }
    }
    let world = frame[end..].to_vec();
    frame.truncate(34);
    frame[16..24].copy_from_slice(&(native.len() as u64).to_le_bytes());
    frame.extend_from_slice(&native);
    frame.extend_from_slice(&world);
    frame
}
