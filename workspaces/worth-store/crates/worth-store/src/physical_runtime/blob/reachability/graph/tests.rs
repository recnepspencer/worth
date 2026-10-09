use std::num::NonZeroU64;

use crate::physical_runtime::blob::reachability::scan::closure::{ClosureFact, TreeLink};
use worth_store_physical_format::BlobTreeNodeKind;
use worth_store_physical_format::{BlobSessionDeclarationV1, PersistedRecordIdentity};

use super::*;
use crate::physical_runtime::blob::reachability::scan::Fact;
use crate::physical_runtime::blob::reachability::BlobReachabilityLimits;

fn id(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

#[test]
fn unreachable_published_session_chunk_needs_release_proof_despite_its_class() {
    let limits = BlobReachabilityLimits::new(
        NonZeroU64::new(16).unwrap(),
        NonZeroU64::new(1024).unwrap(),
        NonZeroU64::new(2).unwrap(),
        NonZeroU64::new(8).unwrap(),
    );
    let mut selected = SelectedInventory::new(limits).unwrap();
    selected.facts.insert(
        id(1),
        Fact {
            current: true,
            held: false,
            role: Role::Publication([3; 16]),
            edges: vec![id(2)],
            declaration: None,
            claim: None,
            closure: Some(ClosureFact::Publication {
                store: [1; 16],
                session: [3; 16],
                root: id(2),
                root_digest: [10; 32],
                frame_digest: [14; 32],
                total_bytes: 64,
                chunk_size: 64,
                scope: [8; 32],
            }),
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(2),
        Fact {
            current: true,
            held: false,
            role: Role::Tree([3; 16]),
            edges: vec![id(3)],
            declaration: None,
            claim: None,
            closure: Some(ClosureFact::Tree {
                store: [1; 16],
                session: [3; 16],
                kind: BlobTreeNodeKind::Leaf,
                level: 0,
                covered_bytes: 64,
                frame_digest: [10; 32],
                canonical_digest: [11; 32],
                links: vec![TreeLink {
                    record: id(3),
                    digest: [12; 32],
                    covered_bytes: 64,
                }],
            }),
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(3),
        Fact {
            current: true,
            held: false,
            role: Role::Chunk {
                session: [3; 16],
                ordinal: 0,
            },
            edges: vec![],
            declaration: None,
            claim: None,
            closure: Some(ClosureFact::Chunk {
                store: [1; 16],
                session: [3; 16],
                ordinal: 0,
                content_digest: [12; 32],
                covered_bytes: 64,
                chunk_size: 64,
                scope: None,
            }),
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(4),
        Fact {
            current: true,
            held: false,
            role: Role::Chunk {
                session: [3; 16],
                ordinal: 1,
            },
            edges: vec![],
            declaration: None,
            claim: None,
            closure: Some(ClosureFact::Chunk {
                store: [1; 16],
                session: [3; 16],
                ordinal: 1,
                content_digest: [13; 32],
                covered_bytes: 64,
                chunk_size: 64,
                scope: None,
            }),
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(5),
        Fact {
            current: true,
            held: false,
            role: Role::Derived,
            edges: vec![],
            declaration: None,
            claim: None,
            closure: None,
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(6),
        Fact {
            current: false,
            held: true,
            role: Role::Chunk {
                session: [4; 16],
                ordinal: 0,
            },
            edges: vec![],
            declaration: None,
            claim: None,
            closure: None,
            control: None,
            reuse: None,
        },
    );
    selected.facts.insert(
        id(7),
        Fact {
            current: true,
            held: true,
            role: Role::Chunk {
                session: [3; 16],
                ordinal: 2,
            },
            edges: vec![],
            declaration: None,
            claim: None,
            closure: None,
            control: None,
            reuse: None,
        },
    );
    for (record, role) in [(id(8), Role::Derived), (id(9), Role::Abandonment([6; 16]))] {
        selected.facts.insert(
            record,
            Fact {
                current: true,
                held: true,
                role,
                edges: vec![],
                declaration: None,
                claim: None,
                closure: None,
                control: None,
                reuse: None,
            },
        );
    }
    selected.edge_count = 2;
    let (rows, _) = classify(&selected).unwrap();
    let class = |ordinal| {
        rows.iter()
            .find(|row| row.record() == id(ordinal))
            .unwrap()
            .class()
    };
    assert_eq!(class(1), BlobRecordReachability::Reachable);
    assert_eq!(class(2), BlobRecordReachability::Reachable);
    assert_eq!(class(3), BlobRecordReachability::Reachable);
    assert_eq!(class(4), BlobRecordReachability::Unreferenced);
    assert_eq!(class(5), BlobRecordReachability::DerivedResidue);
    assert_eq!(class(6), BlobRecordReachability::HeldOnly);
    assert_eq!(class(7), BlobRecordReachability::HeldOnly);
    assert_eq!(class(8), BlobRecordReachability::HeldOnly);
    assert_eq!(class(9), BlobRecordReachability::HeldOnly);
    // Inventory-level lease retirement: the same current routes remain,
    // but the exact protected predecessor no longer owns these records.
    for ordinal in [7, 8, 9] {
        selected.facts.get_mut(&id(ordinal)).unwrap().held = false;
    }
    let (released_rows, _) = classify(&selected).unwrap();
    for (ordinal, expected) in [
        (7, BlobRecordReachability::Unreferenced),
        (8, BlobRecordReachability::DerivedResidue),
        (9, BlobRecordReachability::FailedOperationResidue),
    ] {
        assert_eq!(
            released_rows
                .iter()
                .find(|row| row.record() == id(ordinal))
                .unwrap()
                .class(),
            expected,
        );
    }
    if let Some(ClosureFact::Tree { frame_digest, .. }) =
        &mut selected.facts.get_mut(&id(2)).unwrap().closure
    {
        *frame_digest = [99; 32];
    }
    assert!(matches!(
        classify(&selected),
        Err(Failure::ConflictingSelectedFate)
    ));
}

#[test]
fn selected_unfrontiered_chunk_does_not_become_live_from_session_identity() {
    let limits = BlobReachabilityLimits::new(
        NonZeroU64::new(8).unwrap(),
        NonZeroU64::new(1024).unwrap(),
        NonZeroU64::new(2).unwrap(),
        NonZeroU64::new(8).unwrap(),
    );
    let mut selected = SelectedInventory::new(limits).unwrap();
    for (record, role) in [
        (id(1), Role::Declaration([5; 16])),
        (
            id(2),
            Role::Chunk {
                session: [5; 16],
                ordinal: 0,
            },
        ),
        (id(3), Role::Control),
    ] {
        selected.facts.insert(
            record,
            Fact {
                current: true,
                held: false,
                role,
                edges: vec![],
                declaration: (record == id(1)).then(|| {
                    BlobSessionDeclarationV1::new(
                        [1; 16], [5; 16], [2; 16], [3; 32], 65_536, 65_536, 65_536, 16,
                    )
                    .unwrap()
                }),
                claim: (record == id(2)).then_some(
                    crate::physical_runtime::blob::ingest::SelectedResumeClaim::Chunk {
                        ordinal: 0,
                        record,
                        digest: [4; 32],
                        bytes: 65_536,
                    },
                ),
                closure: None,
                control: None,
                reuse: None,
            },
        );
    }
    let (rows, _) = classify(&selected).unwrap();
    let class = |ordinal| {
        rows.iter()
            .find(|row| row.record() == id(ordinal))
            .unwrap()
            .class()
    };
    assert_eq!(class(1), BlobRecordReachability::Reachable);
    assert_eq!(class(2), BlobRecordReachability::FailedOperationResidue);
    assert_eq!(class(3), BlobRecordReachability::Reachable);

    // A second selected route claiming the same low ordinal cannot be
    // promoted by a frontier or by matching session bytes.
    selected.facts.insert(
        id(4),
        Fact {
            current: true,
            held: false,
            role: Role::Chunk {
                session: [5; 16],
                ordinal: 0,
            },
            edges: vec![],
            declaration: None,
            claim: Some(
                crate::physical_runtime::blob::ingest::SelectedResumeClaim::Chunk {
                    ordinal: 0,
                    record: id(4),
                    digest: [6; 32],
                    bytes: 65_536,
                },
            ),
            closure: None,
            control: None,
            reuse: None,
        },
    );
    assert!(matches!(
        classify(&selected),
        Err(Failure::ConflictingSelectedFate)
    ));
}
