use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::lowering::lower_scope;
use super::{
    effect_relation, PhysicalEffectAccess, PhysicalEffectFootprint, PhysicalEffectKey,
    PhysicalEffectRelation,
};
use crate::physical_runtime::work::PhysicalWorkScope;

fn segment(segment: u64) -> RecordArtifactFile {
    RecordArtifactFile::Segment {
        segment,
        generation: 1,
    }
}

fn footprint(store: u8, keys: Vec<PhysicalEffectKey>) -> PhysicalEffectFootprint {
    PhysicalEffectFootprint::new(store_identity(store), keys)
}

fn store_identity(tag: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([tag; 16]).unwrap(),
    )
    .published_identity()
}

fn range(
    artifact: RecordArtifactFile,
    start: u64,
    end: u64,
    access: PhysicalEffectAccess,
) -> PhysicalEffectKey {
    PhysicalEffectKey::Range {
        artifact,
        start,
        end,
        access,
    }
}

#[test]
fn disjoint_ranges_do_not_conflict() {
    let left = footprint(
        1,
        vec![range(segment(1), 0, 16, PhysicalEffectAccess::Write)],
    );
    let right = footprint(
        1,
        vec![range(segment(2), 0, 16, PhysicalEffectAccess::Write)],
    );
    assert_eq!(
        effect_relation(&left, &right),
        PhysicalEffectRelation::Disjoint
    );
}

#[test]
fn overlapping_writes_conflict_and_overlapping_reads_share() {
    let write = footprint(
        1,
        vec![range(segment(1), 0, 32, PhysicalEffectAccess::Write)],
    );
    let other = footprint(
        1,
        vec![range(segment(1), 16, 48, PhysicalEffectAccess::Write)],
    );
    let read = footprint(
        1,
        vec![range(segment(1), 16, 48, PhysicalEffectAccess::Read)],
    );
    let other_read = footprint(
        1,
        vec![range(segment(1), 0, 32, PhysicalEffectAccess::Read)],
    );
    assert_eq!(
        effect_relation(&write, &other),
        PhysicalEffectRelation::Conflict
    );
    assert_eq!(
        effect_relation(&read, &other_read),
        PhysicalEffectRelation::SharedRead
    );
    assert_eq!(
        effect_relation(&write, &read),
        PhysicalEffectRelation::Conflict
    );
}

#[test]
fn whole_artifact_delete_conflicts_with_any_range_of_that_file() {
    let delete = footprint(
        1,
        vec![PhysicalEffectKey::DeleteArtifact {
            artifact: segment(1),
        }],
    );
    let read = footprint(
        1,
        vec![range(segment(1), 100, 116, PhysicalEffectAccess::Read)],
    );
    let other = footprint(
        1,
        vec![range(segment(2), 100, 116, PhysicalEffectAccess::Read)],
    );
    assert_eq!(
        effect_relation(&delete, &read),
        PhysicalEffectRelation::Conflict
    );
    assert_eq!(
        effect_relation(&delete, &other),
        PhysicalEffectRelation::Disjoint
    );
}

#[test]
fn root_namespace_and_wal_keys_are_not_empty_disjoint() {
    let root = footprint(1, vec![PhysicalEffectKey::RootPublication]);
    let other_root = footprint(1, vec![PhysicalEffectKey::RootPublication]);
    let namespace = footprint(1, vec![PhysicalEffectKey::Namespace]);
    let data = footprint(1, vec![range(segment(1), 0, 8, PhysicalEffectAccess::Read)]);
    let wal = footprint(
        1,
        vec![PhysicalEffectKey::Wal {
            segment: 1,
            generation: 1,
            start: 0,
            end: 64,
            access: PhysicalEffectAccess::Write,
        }],
    );
    let wal_delete = footprint(
        1,
        vec![PhysicalEffectKey::DeleteWal {
            segment: 1,
            generation: 1,
        }],
    );
    let other_wal = footprint(
        1,
        vec![PhysicalEffectKey::Wal {
            segment: 1,
            generation: 1,
            start: 64,
            end: 128,
            access: PhysicalEffectAccess::Write,
        }],
    );
    assert_eq!(
        effect_relation(&root, &other_root),
        PhysicalEffectRelation::Conflict
    );
    assert_eq!(
        effect_relation(&root, &data),
        PhysicalEffectRelation::Disjoint
    );
    assert_eq!(
        effect_relation(&namespace, &data),
        PhysicalEffectRelation::Disjoint
    );
    assert_eq!(
        effect_relation(&wal, &wal_delete),
        PhysicalEffectRelation::Conflict
    );
    assert_eq!(
        effect_relation(&wal, &other_wal),
        PhysicalEffectRelation::Disjoint
    );
}

#[test]
fn different_stores_do_not_share_keys() {
    let left = footprint(1, vec![PhysicalEffectKey::Namespace]);
    let right = footprint(2, vec![PhysicalEffectKey::Namespace]);
    assert_eq!(
        effect_relation(&left, &right),
        PhysicalEffectRelation::Disjoint
    );
}

#[test]
fn scope_lowering_never_emits_an_empty_key_list() {
    let artifact = PhysicalWorkScope::artifact(segment(4));
    let coordinate = RecordFrameCoordinate::new(segment(4), 0, 16).unwrap();
    let one = PhysicalWorkScope::one(coordinate);
    assert!(matches!(
        lower_scope(&artifact, PhysicalEffectAccess::Write)[..],
        [PhysicalEffectKey::WholeArtifact { .. }]
    ));
    let removal = PhysicalWorkScope::artifact_removal(segment(4));
    assert!(matches!(
        lower_scope(&removal, PhysicalEffectAccess::Write)[..],
        [PhysicalEffectKey::DeleteArtifact { .. }]
    ));
    assert!(matches!(
        lower_scope(&one, PhysicalEffectAccess::Read)[..],
        [PhysicalEffectKey::Range { .. }]
    ));
    let free = PhysicalWorkScope::artifact(RecordArtifactFile::FreeSpaceManifest { generation: 3 });
    assert!(matches!(
        lower_scope(&free, PhysicalEffectAccess::Write)[..],
        [PhysicalEffectKey::Allocator {
            generation: 3,
            block: None
        }]
    ));
}
