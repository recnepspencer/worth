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
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let frozen = selections.freeze_for_request(&request).unwrap();
    assert_eq!(frozen.0.len(), 2);
    assert!(frozen.rostered(&roster));
    assert!(!frozen.rostered(&roster[..1]));
    let resolved = frozen.resolve(&carried, &roster, &request).unwrap();
    assert_eq!(
        resolved.rows(),
        &[
            image(2, 2, &[(1, 1), (2, 2)], &[]),
            image(5, 5, &[(1, 3), (2, 2)], &[3]),
            image(6, 6, &[], &[2]),
        ]
    );
    // Without an image the live selections travel unchanged.
    let alone = selections
        .freeze_for_request(&request)
        .unwrap()
        .resolve(&[], &roster, &request)
        .unwrap();
    assert_eq!(
        alone.rows(),
        &[image(5, 5, &[(1, 3)], &[]), image(6, 6, &[], &[2])]
    );
}

#[test]
fn a_selection_whose_generation_is_not_emitted_refuses_capture() {
    let roster = [generation(1, 1, 2)];
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let mut selections = BasisSelections::default();
    selections.install(live(5, 5, &[&generation(1, 3, 5)], &[]));
    for (selections, carried) in [
        (selections, vec![]),
        (
            BasisSelections::default(),
            vec![image(2, 2, &[(1, 9)], &[])],
        ),
    ] {
        let frozen = selections.freeze_for_request(&request).unwrap();
        assert!(matches!(
            frozen.resolve(&carried, &roster, &request),
            Err(Denial::OriginalAuthorityUnavailable)
        ));
    }
}
