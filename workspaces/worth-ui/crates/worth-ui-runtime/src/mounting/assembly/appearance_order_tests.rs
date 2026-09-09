use super::UiPreparedMountedFrame;
use crate::mounting::projection::{
    UiMountedAppearanceOrderDenial, UiMountedAppearanceOutputDenial,
};
use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPresentationAttemptIdentity, UiSemanticSurfaceIdentity,
};

impl UiPreparedMountedFrame {
    pub(crate) fn verify_retained_order_conflict(
        &self,
        surface: UiSemanticSurfaceIdentity,
        retained: UiMountedInstanceIdentity,
        arriving: UiMountedInstanceIdentity,
    ) {
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        let previous = owner.appearance().physical_node_receipts_for_test();
        assert_eq!(
            previous.len(),
            2,
            "both surfaces already retain their own appearance"
        );
        assert!(previous
            .iter()
            .any(|receipt| receipt.mounted_instance() == retained));
        let attempts = owner.appearance().pending_attempts_for_test();
        assert_eq!(
            attempts.len(),
            1,
            "the conflicting neighbor is retained and unselected"
        );
        assert_eq!(attempts[0].context().mounted_instance(), arriving);
        let bytes = owner.appearance_order_retained_bytes();
        assert!(bytes > 0);
        assert_eq!(
            owner
                .appearance_selection_cost_report()
                .order_retained_bytes(),
            bytes
        );
        let records = owner.lower_appearance(
            UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            self.manifest.surfaces(),
            None,
        );
        assert_eq!(records.len(), 1);
        assert_eq!(
            owner.unpublished_appearance().unwrap_err(),
            &UiMountedAppearanceOutputDenial::Order(UiMountedAppearanceOrderDenial::Ambiguous {
                surface,
                first: retained,
                second: arriving,
                rank: 65_536,
            })
        );
        assert_eq!(
            owner.appearance().physical_node_receipts_for_test(),
            previous
        );
        assert_eq!(owner.appearance_order_retained_bytes(), bytes);
        // Retrying the real attempted contribution must still find the retained
        // neighbor. A leaked removal on the first failure would make this pass.
        owner
            .stage_appearance_projection(attempts[0].clone())
            .unwrap();
        owner.lower_appearance(
            UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            self.manifest.surfaces(),
            None,
        );
        assert!(matches!(
            owner.unpublished_appearance(),
            Err(UiMountedAppearanceOutputDenial::Order(
                UiMountedAppearanceOrderDenial::Ambiguous { .. }
            ))
        ));
        assert_eq!(
            owner.appearance().physical_node_receipts_for_test(),
            previous
        );
        assert_eq!(owner.appearance_order_retained_bytes(), bytes);
    }
}
