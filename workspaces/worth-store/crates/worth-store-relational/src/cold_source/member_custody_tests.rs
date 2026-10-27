//! The real checkpoint publisher preserves only live placement obligations.
//! The native recovery basis is producer-written; the extra opaque member
//! isolates placement lifetime from Relational record admission.
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::RelationalCheckpointArtifactPolicy;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordPublication, PhysicalMutationDeadline,
};

#[test]
fn checkpoint_replacement_carries_a_member_until_its_last_read_custody_drops() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(
        crate::physical_store::initialize(
            directory.path(),
            crate::physical_store::PhysicalOpening::production(),
        )
        .unwrap(),
    );
    let source = StoreColdSource::new(serving.clone());
    let runtime = RelationalRuntimeApi::builder().build();
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let replace = || {
        crate::checkpoint_record::publish(
            &serving,
            &source,
            PatchStreamPosition(0),
            RelationalBranchTransitionSequence(0),
            &native,
        )
        .unwrap()
    };
    let initial = replace();
    let records = serving.anchored_records().unwrap();
    let anchor = AnchoredRecordName::new("relational.checkpoint").unwrap();
    let previous = records.read(anchor, 1 << 20).unwrap().unwrap();
    let bytes = b"retained older history placement".to_vec();
    let locator = RelationalDurableMemberLocator {
        sha256: Sha256::digest(&bytes).into(),
        length: bytes.len() as u64,
    };
    let published = records
        .publish(
            AnchoredRecordPublication::new(
                anchor,
                initial,
                previous.bytes().to_vec(),
                crate::physical_store::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(
                previous
                    .members()
                    .iter()
                    .map(|locator| AnchoredRecordMember::Carried { locator: *locator })
                    .chain(std::iter::once(AnchoredRecordMember::New {
                        key: locator.sha256,
                        bytes: bytes.clone(),
                    }))
                    .collect(),
            ),
        )
        .unwrap();
    source.select_members(published.version().generation(), published.members());

    let before = source.counters();
    let custody = source.retain_members(&[locator, locator]).unwrap();
    let last = custody.clone();
    assert_eq!(
        source.counters(),
        before,
        "retention never reads member bytes"
    );
    replace();
    assert_eq!(source.read_member(locator, 1 << 20).unwrap(), bytes);
    drop(custody);
    replace();
    assert_eq!(source.read_member(locator, 1 << 20).unwrap(), bytes);
    drop(last);
    replace();
    assert_eq!(
        source.read_member(locator, 1 << 20),
        Err(Cause::MissingMember)
    );
    assert!(matches!(
        source.retain_members(&[locator]),
        Err(Cause::MissingMember)
    ));
    let remaining = source.member_locators();
    assert!(
        !remaining.is_empty(),
        "the producer-written native basis still has its index member"
    );
    let provider_owner = source.retain_members(&remaining).unwrap();
    drop(records);
    drop(source);
    let serving = match Arc::try_unwrap(serving) {
        Err(serving) => serving,
        Ok(_) => panic!("member custody must keep its provider's physical session alive"),
    };
    drop(provider_owner);
    let serving = Arc::try_unwrap(serving)
        .ok()
        .expect("all placement owners released");
    let _closed = serving.close();
}
