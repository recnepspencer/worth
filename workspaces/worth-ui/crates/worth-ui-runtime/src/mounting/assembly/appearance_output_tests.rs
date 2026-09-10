use crate::runtime::appearance::UiAppearanceInspectionRecord;

impl super::UiPreparedMountedFrame {
    pub(crate) fn assert_no_unpublished_appearance_for_test(&self) {
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert!(records.is_empty());
        assert!(owner.unpublished_appearance().unwrap().is_none());
    }

    pub(crate) fn lower_unpublished_appearance_for_test(
        &self,
    ) -> worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection {
        self.lower_unpublished_appearance_with_profile_for_test(None)
    }

    pub(crate) fn lower_unpublished_appearance_with_profile_for_test(
        &self,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection {
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), profile);
        assert!(
            records.iter().all(|record| matches!(
                record,
                crate::runtime::appearance::UiAppearanceInspectionRecord::Projection { .. }
            )),
            "appearance output denied: {:?}",
            owner.unpublished_appearance().err(),
        );
        owner.unpublished_appearance().unwrap().unwrap().clone()
    }

    pub(crate) fn qualified_outline_fringe_for_test(
        &self,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<
        worth_ui_host_contract::UiAppearanceLogicalLength,
        crate::mounting::UiMountedAppearanceLoweringDenial,
    > {
        self.candidate.owner.qualified_outline_fringe_for_test(
            self.manifest.surfaces(),
            profile,
            surface,
        )
    }

    pub(crate) fn verify_outline_geometry_denial_preserves_predecessor(
        &self,
        profile: &worth_ui_host_contract::UiHostAppearanceProfileContract,
    ) {
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        let predecessors = owner.appearance().physical_node_receipts_for_test();
        assert!(!predecessors.is_empty());
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert!(!records.is_empty());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(
            owner.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::HostGeometryProfileUnavailable
        );
        assert_eq!(
            owner.appearance().physical_node_receipts_for_test(),
            predecessors
        );
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), Some(profile));
        assert!(!records.is_empty());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Projection { .. })));
        assert!(owner.unpublished_appearance().unwrap().is_some());
    }

    pub(crate) fn verify_unpublished_reconstruction_denial_and_retry(&self) {
        self.verify_unpublished_denial_and_retry(
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction,
        );
    }

    pub(crate) fn verify_unpublished_appearance_retirement_denial_and_retry(
        &self,
        retiring_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) {
        let mut rejected = self.candidate.owner.clone_for_appearance_output_test();
        let mut control = self.candidate.owner.clone_for_appearance_output_test();
        let predecessors = rejected.appearance().physical_node_receipts_for_test();
        assert!(predecessors.contains(&retiring_receipt));
        let attempts = rejected.appearance().pending_attempts_for_test();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = rejected.lower_appearance(presentation, &[], None);
        assert_eq!(records.len(), attempts.len());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(rejected.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::Transport(
                worth_ui_host_contract::UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch));
        assert_eq!(
            rejected.appearance().physical_node_receipts_for_test(),
            predecessors
        );
        for attempt in attempts {
            rejected.stage_appearance_projection(attempt).unwrap();
        }
        rejected.lower_appearance(presentation, self.manifest.surfaces(), None);
        control.lower_appearance(presentation, self.manifest.surfaces(), None);
        let retry = rejected.unpublished_appearance().unwrap().unwrap();
        assert_eq!(Some(retry), control.unpublished_appearance().unwrap());
        assert!(retry.fragments().iter().any(|fragment| fragment.identity()
            == worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: Some(retiring_receipt),
                successor: None,
            }));
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(retry).unwrap();
    }

    pub(crate) fn verify_mixed_appearance_reconstruction_denial_and_retry(&self) {
        let mut rejected = self.candidate.owner.clone_for_appearance_output_test();
        let mut control = self.candidate.owner.clone_for_appearance_output_test();
        let attempts = rejected.appearance().pending_attempts_for_test();
        let predecessors = rejected.appearance().physical_node_receipts_for_test();
        assert_eq!(
            attempts.len(),
            1,
            "one current resolver result exercises staged reconstruction"
        );
        assert!(
            predecessors.len() > 1,
            "other nodes exercise retained reconstruction"
        );
        rejected.prepare_appearance_reconstruction().unwrap();
        control.prepare_appearance_reconstruction().unwrap();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = rejected.lower_appearance(presentation, &[], None);
        assert_eq!(records.len(), predecessors.len());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(rejected.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::Transport(
                worth_ui_host_contract::UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch));
        assert_eq!(
            rejected.appearance().physical_node_receipts_for_test(),
            predecessors
        );
        for attempt in attempts {
            rejected.stage_appearance_projection(attempt).unwrap();
        }
        let records = rejected.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert_eq!(records.len(), predecessors.len());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Projection { .. })));
        control.lower_appearance(presentation, self.manifest.surfaces(), None);
        let output = rejected.unpublished_appearance().unwrap().unwrap();
        assert_eq!(Some(output), control.unpublished_appearance().unwrap());
        assert_eq!(output.fragments().len(), predecessors.len());
        let mut actual_predecessors = Vec::new();
        let mut successors = std::collections::BTreeSet::new();
        let mut changed = 0;
        for fragment in output.fragments() {
            let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: Some(previous),
                successor: Some(next),
            } = fragment.identity()
            else {
                panic!("every rebuilt node has exact physical predecessors");
            };
            actual_predecessors.push(previous);
            assert!(
                successors.insert(next),
                "a node may be reconstructed only once"
            );
            assert_eq!(
                fragment.work().posture(),
                worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
            );
            changed += usize::from(!fragment.work().changes().is_empty());
        }
        actual_predecessors.sort();
        assert_eq!(actual_predecessors, predecessors);
        assert_eq!(
            changed, 1,
            "only the selected theme consumer changes mechanics"
        );
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    }

    pub(crate) fn verify_unpublished_appearance_member_denial(&self) {
        let original_cost = self.candidate.owner.appearance_selection_cost_report();
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        let attempts = owner.appearance().pending_attempts_for_test();
        assert!(
            attempts.len() > 1,
            "partial-output regression requires several real consumers"
        );
        let predecessor_receipts = owner.appearance().physical_node_receipts_for_test();
        owner
            .stage_appearance_projection(
                crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                    attempts[0].context().clone(),
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                ),
            )
            .unwrap();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert_eq!(records.len(), attempts.len());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(
            owner.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::NodeLowering
        );
        assert_eq!(
            owner.appearance().physical_node_receipts_for_test(),
            predecessor_receipts
        );
        for attempt in &attempts {
            owner.stage_appearance_projection(attempt.clone()).unwrap();
        }
        let records = owner.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Projection { .. })));
        assert_eq!(
            owner
                .unpublished_appearance()
                .unwrap()
                .unwrap()
                .fragments()
                .len(),
            attempts.len()
        );
        assert_eq!(
            self.candidate.owner.appearance_selection_cost_report(),
            original_cost
        );
    }

    pub(crate) fn verify_unpublished_appearance_denial_and_retry(&self) {
        self.verify_unpublished_denial_and_retry(
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta,
        );
        self.candidate
            .owner
            .verify_appearance_suppression_lifecycle_for_test(self.manifest.surfaces());
    }

    fn verify_unpublished_denial_and_retry(
        &self,
        expected_posture: worth_ui_host_contract::UiMountedAppearanceWorkPosture,
    ) {
        let original_cost = self.candidate.owner.appearance_selection_cost_report();
        let mut rejected = self.candidate.owner.clone_for_appearance_output_test();
        let mut control = self.candidate.owner.clone_for_appearance_output_test();
        let attempts = rejected.appearance().pending_attempts_for_test();
        let predecessor_receipts = rejected.appearance().physical_node_receipts_for_test();
        assert!(
            !attempts.is_empty(),
            "scenario must stage real resolved work"
        );
        assert!(
            !predecessor_receipts.is_empty(),
            "scenario needs an established predecessor"
        );
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let records = rejected.lower_appearance(presentation, &[], None);
        assert_eq!(records.len(), attempts.len());
        assert!(records
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Denial { .. })));
        assert_eq!(rejected.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::Transport(
                worth_ui_host_contract::UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch));
        assert_eq!(
            rejected.appearance().physical_node_receipts_for_test(),
            predecessor_receipts
        );
        for attempt in attempts {
            rejected.stage_appearance_projection(attempt).unwrap();
        }
        let retry = rejected.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert!(retry
            .iter()
            .all(|record| matches!(record, UiAppearanceInspectionRecord::Projection { .. })));
        control.lower_appearance(presentation, self.manifest.surfaces(), None);
        let retry_output = rejected.unpublished_appearance().unwrap().unwrap();
        assert_eq!(
            Some(retry_output),
            control.unpublished_appearance().unwrap()
        );
        assert!(retry_output.fragments().iter().all(|fragment| {
            fragment.work().posture() == expected_posture && fragment.work().predecessor().is_some()
        }));
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(retry_output)
            .unwrap();
        assert_eq!(
            self.candidate.owner.appearance_selection_cost_report(),
            original_cost
        );
    }
}
