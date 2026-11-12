//! Pure selection bookkeeping; these values claim no runtime or commit authority.
use super::super::*;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexGenerationId,
    DerivedIndexPublicationStatus,
};

fn selected(value: u8, id: u64) -> PreparedIndexBasisSelection {
    let index = DerivedIndexId(1);
    let mut basis = PreparedIndexBasisSelection {
        branch: BranchId("selection-values".into()),
        commit: CommitId(3),
        version: VersionId(2),
        schema: SchemaVersionId(1),
        state: Arc::default(),
        globals: std::iter::once(index).collect(),
    };
    match value {
        0 => {}
        1 => basis.replace(Arc::new(DerivedIndexGeneration {
            index_id: index,
            generation_id: DerivedIndexGenerationId(id),
            source_commit_id: basis.commit,
            source_branch_id: basis.branch.clone(),
            applicability: DerivedIndexApplicability {
                branch_id: basis.branch.clone(),
                version_id: basis.version,
                schema_version: basis.schema,
            },
            status: DerivedIndexPublicationStatus::Published,
            entries: DerivedIndexEntries::EntityField(Default::default()),
        })),
        2 => {
            let state = Arc::make_mut(&mut basis.state);
            state.empty.insert(index);
            state.global_empty.insert(index);
        }
        _ => panic!("three bookkeeping states"),
    }
    basis
}

#[test]
fn deferred_basis_merge_preserves_live_entries_empty_bindings_and_retained_pins() {
    let index = DerivedIndexId(1);
    for live_state in 0..3 {
        for image_state in 0..3 {
            let live = selected(live_state, 9);
            let mut current = BasisSelections::default();
            current.install(live.clone());
            let retained = current
                .get(Some(&live.branch), live.version)
                .unwrap()
                .clone();
            let restored = selected(image_state, 4);
            let mut image = BasisSelections::default();
            image.install(restored);
            current.merge(&image);
            let expected = if live_state == 0 {
                image_state
            } else {
                live_state
            };
            let id = if live_state == 1 { 9 } else { 4 };
            for branch in [Some(&live.branch), None] {
                let basis = current.get(branch, live.version);
                if expected == 0 {
                    continue;
                }
                let state = &basis.unwrap().state;
                assert_eq!(
                    state.entries.get(&index).map(|g| g.generation_id),
                    (expected == 1).then_some(DerivedIndexGenerationId(id))
                );
                assert_eq!(state.empty.contains(&index), expected == 2);
                assert_eq!(
                    state.global.get(&index).map(|g| g.generation_id),
                    state.entries.get(&index).map(|g| g.generation_id)
                );
                assert_eq!(
                    state.global_empty.contains(&index),
                    state.empty.contains(&index)
                );
            }
            assert_eq!(
                retained.state.entries.get(&index).map(|g| g.generation_id),
                (live_state == 1).then_some(DerivedIndexGenerationId(9))
            );
            assert_eq!(retained.state.empty.contains(&index), live_state == 2);
        }
    }
}
