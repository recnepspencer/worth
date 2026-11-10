//! Pure selection rows; these values claim no runtime or commit authority.
use super::*;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexPublicationStatus,
};
use crate::schema::data::SchemaVersionId;

fn generation(index: u64, id: u64, version: u64) -> Arc<DerivedIndexGeneration> {
    Arc::new(DerivedIndexGeneration {
        index_id: DerivedIndexId(index),
        generation_id: DerivedIndexGenerationId(id),
        source_commit_id: CommitId(version),
        source_branch_id: BranchId("main".into()),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId("main".into()),
            version_id: VersionId(version),
            schema_version: SchemaVersionId(1),
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(Default::default()),
    })
}
fn live(
    version: u64,
    commit: u64,
    bound: &[&Arc<DerivedIndexGeneration>],
    empty: &[u64],
) -> PreparedIndexBasisSelection {
    let mut basis = PreparedIndexBasisSelection {
        branch: BranchId("main".into()),
        commit: CommitId(commit),
        version: VersionId(version),
        schema: SchemaVersionId(1),
        state: Arc::default(),
        // A global index also installs a global lookup, which is never frozen.
        globals: std::iter::once(DerivedIndexId(1)).collect(),
    };
    for generation in bound {
        basis.replace(Arc::clone(generation));
    }
    let state = Arc::make_mut(&mut basis.state);
    for index in empty {
        state.empty.insert(DerivedIndexId(*index));
    }
    basis
}
fn image(
    version: u64,
    commit: u64,
    bindings: &[(u64, u64)],
    empty: &[u64],
) -> IndexBasisCheckpoint {
    IndexBasisCheckpoint {
        branch: BranchId("main".into()),
        commit: CommitId(commit),
        version: VersionId(version),
        schema: SchemaVersionId(1),
        empty: empty.iter().copied().map(DerivedIndexId).collect(),
        bindings: bindings
            .iter()
            .map(|(index, id)| (DerivedIndexId(*index), DerivedIndexGenerationId(*id)))
            .collect(),
    }
}

/// The pure row fold `resolve` performs after it funds each row.
fn rows(
    live: &[Arc<PreparedIndexBasisSelection>],
    carried: &[IndexBasisCheckpoint],
    roster: &[Arc<DerivedIndexGeneration>],
) -> Result<Vec<IndexBasisCheckpoint>, Denial> {
    Paired::new(live, carried)
        .map(|(live, image)| portable(live, admitted_image(live, image), roster))
        .collect()
}

#[test]
fn frozen_selections_keep_live_bindings_and_fill_only_what_a_deferred_image_carried() {
    let (old, other, new) = (
        generation(1, 1, 2),
        generation(2, 2, 2),
        generation(1, 3, 5),
    );
    // The roster order of every Complete capture: `(index, generation)`.
    let roster = [Arc::clone(&old), Arc::clone(&new), Arc::clone(&other)];
    let mut selections = BasisSelections::default();
    selections.install(live(5, 5, &[&new], &[]));
    selections.install(live(6, 6, &[], &[2]));
    let carried = [
        // No live selection names this root: the image row is kept whole.
        image(2, 2, &[(1, 1), (2, 2)], &[]),
        // Same root: the live binding wins, the other selections are filled.
        image(5, 5, &[(1, 1), (2, 2)], &[3]),
        // Another commit at a live key is stale and contributes nothing.
        image(6, 60, &[(1, 1)], &[3]),
    ];
    // The global lookup the index-1 selection installed is never frozen.
    let frozen: Vec<_> = selections.branch_selections().cloned().collect();
    assert_eq!(frozen.len(), 2);
    assert!(rostered(&frozen, &roster));
    assert!(!rostered(&frozen, &roster[..1]));
    assert_eq!(
        rows(&frozen, &carried, &roster).unwrap(),
        [
            image(2, 2, &[(1, 1), (2, 2)], &[]),
            image(5, 5, &[(1, 3), (2, 2)], &[3]),
            image(6, 6, &[], &[2]),
        ]
    );
    // Without an image the live selections travel unchanged.
    assert_eq!(
        rows(&frozen, &[], &roster).unwrap(),
        [image(5, 5, &[(1, 3)], &[]), image(6, 6, &[], &[2])]
    );
    // Each row is funded, before it is built, for exactly the capacity it is built with.
    let binding = std::mem::size_of::<(DerivedIndexId, DerivedIndexGenerationId)>() as u64;
    let selected = std::mem::size_of::<DerivedIndexId>() as u64;
    let funded: Vec<_> = Paired::new(&frozen, &carried)
        .map(|(live, carried)| row_bytes(live, admitted_image(live, carried)).unwrap())
        .collect();
    let branch = "main".len() as u64;
    assert_eq!(
        funded,
        [
            2 * binding + branch,
            3 * binding + selected + branch,
            selected + branch
        ]
    );
    for (live, carried) in Paired::new(&frozen, &carried) {
        let carried = admitted_image(live, carried);
        let row = portable(live, carried, &roster).unwrap();
        let (bindings, empty) = capacities(live, carried).unwrap();
        assert!(row.bindings.capacity() == bindings && row.empty.capacity() == empty);
    }
}

#[test]
fn a_selection_whose_generation_is_not_emitted_refuses_capture() {
    let roster = [generation(1, 1, 2)];
    let mut selections = BasisSelections::default();
    selections.install(live(5, 5, &[&generation(1, 3, 5)], &[]));
    let frozen: Vec<_> = selections.branch_selections().cloned().collect();
    for (live, carried) in [
        (frozen.as_slice(), vec![]),
        (&[][..], vec![image(2, 2, &[(1, 9)], &[])]),
    ] {
        assert!(matches!(
            rows(live, &carried, &roster),
            Err(Denial::OriginalAuthorityUnavailable)
        ));
    }
}
