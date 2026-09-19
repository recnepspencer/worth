use super::super::state_test_fixture::{incarnation, key, owner, registration};
use super::super::*;
use crate::runtime::persistent_index::{begin_all_test_observation, test_work};

#[test]
fn selection_snapshot_lookup_and_key_comparison_scale_with_changed_paths() {
    // Separate owner-count and collection-size axes. This is owner-storage
    // evidence, not a claim about mounted row binding or native frame cost.
    for (owner_count, key_count) in [(8usize, 64usize), (512, 64), (8, 4_096)] {
        let first = owner();
        let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
        let keys = (1..=key_count as u64).map(key).collect::<Vec<_>>();
        let mut owners = Vec::new();
        for index in 0..owner_count {
            let owner = UiSelectionOwnerIdentity::new(
                first.semantic_surface(),
                crate::graph::UiGraphNodeIdentity::new(100 + index as u64),
                first.key_family(),
            );
            state
                .synchronize(registration(
                    owner,
                    UiSelectionPolicy::MultipleWithRange,
                    keys.clone(),
                    UiSelectionCatalogPosture::Complete,
                ))
                .unwrap();
            state
                .apply(
                    owner,
                    incarnation(),
                    UiSelectionRequest::SelectSingle(keys[0]),
                )
                .unwrap();
            state
                .apply(
                    owner,
                    incarnation(),
                    UiSelectionRequest::SelectRange {
                        target: *keys.last().unwrap(),
                        extend: true,
                    },
                )
                .unwrap();
            owners.push(owner);
        }
        let target = owners[owner_count / 2];
        let changed_key = keys[key_count / 2];
        begin_all_test_observation();
        let before = state.appearance_owner_snapshot();
        assert_eq!(before, before.clone());
        assert_eq!(test_work().lookup_probes(), 0);
        assert_eq!(test_work().iterated_entries(), 0);
        let prior = before
            .posture_for(target, changed_key, incarnation())
            .unwrap();
        assert_eq!(prior.source_bits(), (true, false, false));
        assert!(
            test_work().lookup_probes() <= 4 * (owner_count.ilog2() + key_count.ilog2()) as usize
        );
        assert_eq!(test_work().iterated_entries(), 0);

        begin_all_test_observation();
        let delta = state
            .apply(
                target,
                incarnation(),
                UiSelectionRequest::Remove(changed_key),
            )
            .unwrap();
        assert_eq!(delta.removed(), &[changed_key]);
        let after = state.appearance_owner_snapshot();
        assert_eq!(
            after.changes_since(&before),
            vec![UiSelectionAppearanceChange::Keys {
                owner: target,
                incarnation: incarnation(),
                keys: vec![changed_key].into(),
            }]
        );
        let work = test_work();
        assert_eq!(
            work.iterated_entries(),
            0,
            "ordinary change must not traverse selected sets or catalogs"
        );
        assert!(
            work.comparison_steps() <= 12 * (owner_count.ilog2() + key_count.ilog2() + 2) as usize
        );
        assert_eq!(
            before
                .posture_for(target, changed_key, incarnation())
                .unwrap(),
            prior
        );
        assert_eq!(
            after
                .posture_for(target, changed_key, incarnation())
                .unwrap()
                .source_bits(),
            (false, false, false)
        );
        assert_eq!(
            before.posture_for(owners[0], changed_key, incarnation()),
            after.posture_for(owners[0], changed_key, incarnation()),
            "unrelated owner's evidence remains unchanged"
        );
    }
}
