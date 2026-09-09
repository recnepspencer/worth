use super::UiMountedProjectionFrameOwner;
use crate::runtime::appearance::{UiAppearanceInspectionRecord, UiAppearanceProjectionAttempt};
use worth_ui_host_contract::{
    UiMountedAppearanceMechanicChange, UiMountedPresentationAttemptIdentity,
    UiMountedSurfaceBindingRequirement, UiUnpublishedAppearanceFragmentIdentity,
};

impl UiMountedProjectionFrameOwner {
    /// Existing real mounted session tests supply current resolved attempts.
    /// Only clip posture is injected here: this proves aggregate state lifecycle,
    /// while Portal geometry production is tested at the mounted completion owner.
    pub(crate) fn verify_appearance_suppression_lifecycle_for_test(
        &self,
        bindings: &[UiMountedSurfaceBindingRequirement],
    ) {
        let mut owner = self.clone_for_appearance_output_test();
        let attempts = owner.appearance.pending_attempts_for_test();
        assert!(!attempts.is_empty());
        let presentation = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        assert_projections(
            &owner.lower_appearance(presentation, bindings, None),
            attempts.len(),
        );
        let visible = owner.unpublished_appearance().unwrap().unwrap().clone();
        let predecessors = owner.appearance.physical_node_receipts_for_test();
        let membership = owner.appearance.membership_counts();
        assert!(membership.0 >= attempts.len());
        assert_eq!((membership.1, membership.2), (0, 0));
        let suppressed = attempts
            .iter()
            .map(|attempt| {
                UiAppearanceProjectionAttempt::resolved(
                    attempt
                        .context()
                        .clone()
                        .with_clip_for_test(crate::mounting::UiMountedAppearanceClip::Suppressed),
                    attempt.projection().unwrap().clone(),
                )
            })
            .collect::<Vec<_>>();

        stage(&mut owner, &suppressed);
        let denied = owner.lower_appearance(presentation, &[], None);
        assert_eq!(denied.len(), attempts.len());
        assert!(denied
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(owner.unpublished_appearance().unwrap_err(),
            &super::UiMountedAppearanceOutputDenial::Transport(
                worth_ui_host_contract::UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch));
        assert_eq!(
            owner.appearance.physical_node_receipts_for_test(),
            predecessors
        );
        assert_eq!(owner.appearance.membership_counts(), membership);

        stage(&mut owner, &suppressed);
        assert_projections(
            &owner.lower_appearance(presentation, bindings, None),
            attempts.len(),
        );
        let removal = owner.unpublished_appearance().unwrap().unwrap();
        let removed_nodes = removal
            .fragments()
            .iter()
            .filter_map(|fragment| match fragment.identity() {
                UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                    predecessor: Some(previous),
                    successor: None,
                } => {
                    assert!(fragment.work().successor().mechanics().is_empty());
                    assert!(!fragment.work().changes().is_empty());
                    assert!(fragment.work().changes().iter().all(|change| matches!(
                        change,
                        UiMountedAppearanceMechanicChange::Remove(_)
                    )));
                    Some(previous.mounted_instance())
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            removed_nodes,
            attempts
                .iter()
                .map(|attempt| attempt.context().mounted_instance())
                .collect()
        );
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(removal)
            .unwrap();
        assert_eq!(
            owner.appearance.membership_counts(),
            membership,
            "suppression retains capacity-counted semantic membership"
        );
        for attempt in &attempts {
            let entry = owner
                .appearance
                .retained_entry_for_test(attempt.context())
                .unwrap();
            assert_eq!(&entry.projection, attempt.projection().unwrap());
            assert!(entry.sidecar.current_node_receipts().is_empty());
        }

        stage(&mut owner, &suppressed);
        assert_projections(
            &owner.lower_appearance(presentation, bindings, None),
            attempts.len(),
        );
        assert!(owner.unpublished_appearance().unwrap().is_none());
        assert_eq!(owner.appearance.membership_counts(), membership);

        stage(&mut owner, &attempts);
        assert_projections(
            &owner.lower_appearance(presentation, bindings, None),
            attempts.len(),
        );
        let restored = owner.unpublished_appearance().unwrap().unwrap();
        let mut restored_nodes = std::collections::BTreeSet::new();
        for fragment in restored.fragments() {
            if let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: None,
                successor: Some(receipt),
            } = fragment.identity()
            {
                restored_nodes.insert(receipt.mounted_instance());
                let before = visible.fragments().iter().find(|previous| matches!(previous.identity(),
                    UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor: Some(previous), .. } if previous == receipt
                )).unwrap();
                assert_eq!(
                    fragment.work().successor().mechanics(),
                    before.work().successor().mechanics()
                );
            }
        }
        assert_eq!(restored_nodes, removed_nodes);
        assert_eq!(
            owner.appearance.physical_node_receipts_for_test(),
            predecessors
        );
        assert_eq!(owner.appearance.membership_counts(), membership);
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(restored)
            .unwrap();
    }
}

fn stage(owner: &mut UiMountedProjectionFrameOwner, attempts: &[UiAppearanceProjectionAttempt]) {
    for attempt in attempts {
        owner.stage_appearance_projection(attempt.clone()).unwrap();
    }
}

fn assert_projections(records: &[UiAppearanceInspectionRecord], expected: usize) {
    assert_eq!(records.len(), expected);
    assert!(records
        .iter()
        .all(|record| matches!(record, UiAppearanceInspectionRecord::Projection { .. })));
}
