use worth_ui_host_contract::{
    UiMountedFilledRectMechanic, UiMountedFrameIdentity, UiMountedHitTestMechanic,
    UiMountedSemanticTextMechanic, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use super::UiMountedMechanicSource;
use crate::mounting::projection::frame_storage::UiMountedSemanticProjection;
use crate::mounting::{UiMountedNodeReceiptBasis, UiMountedProjectionDenial};

impl UiMountedMechanicSource {
    pub(in crate::mounting::projection::frame_storage) fn semantic_text_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        frame: UiMountedFrameIdentity,
        receipts: &UiMountedNodeReceiptBasis,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedProjectionDenial> {
        let receipt = receipt_for(
            receipts,
            instance,
            UiMountedProjectionDenial::SemanticTextNodeReceiptMismatch,
        )?;
        self.semantic_text
            .retained_rows_for_instance(instance)
            .map(|row| row.reattributed_mechanic(content, frame, receipt))
            .collect()
    }

    /// Allocation-space candidates only; presented Portal/Motion geometry is downstream.
    pub(in crate::mounting::projection::frame_storage) fn allocation_hit_candidates(
        &self,
        binding: UiSurfaceBindingGeneration,
        space: worth_ui_host_contract::UiMountedCoordinateSpace,
        point: [f64; 2],
        budget: crate::mounting::spatial_index::UiMountedSpatialBudget,
    ) -> Result<
        crate::mounting::spatial_index::UiMountedSpatialQuery,
        crate::mounting::spatial_index::UiMountedSpatialQueryDenial,
    > {
        self.hit_tests
            .allocation_candidates(binding, space, point, budget)
    }

    pub(in crate::mounting::projection::frame_storage) fn filled_rects_for(
        &self,
        semantic: &UiMountedSemanticProjection,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        frame: UiMountedFrameIdentity,
        receipts: &UiMountedNodeReceiptBasis,
    ) -> Result<Vec<UiMountedFilledRectMechanic>, UiMountedProjectionDenial> {
        semantic
            .order
            .iter()
            .filter_map(|instance| self.filled_rects.get(instance).copied())
            .filter(|row| row.surface() == surface && row.binding() == binding)
            .map(|row| {
                crate::mounting::projection::static_paint::reattribute_filled_rect(
                    row, frame, receipts,
                )
            })
            .collect()
    }

    pub(in crate::mounting::projection::frame_storage) fn semantic_text_for(
        &self,
        semantic: &UiMountedSemanticProjection,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        frame: UiMountedFrameIdentity,
        receipts: &UiMountedNodeReceiptBasis,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedProjectionDenial> {
        semantic
            .order
            .iter()
            .flat_map(|instance| self.semantic_text.retained_rows_for_instance(*instance))
            .filter(|row| row.surface() == surface && row.binding() == binding)
            .map(|row| {
                let receipt = receipt_for(
                    receipts,
                    row.mounted_instance(),
                    UiMountedProjectionDenial::SemanticTextNodeReceiptMismatch,
                )?;
                row.reattributed_mechanic(content, frame, receipt)
            })
            .collect()
    }

    pub(in crate::mounting::projection::frame_storage) fn hit_tests_for(
        &self,
        semantic: &UiMountedSemanticProjection,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        frame: UiMountedFrameIdentity,
        receipts: &UiMountedNodeReceiptBasis,
    ) -> Result<Vec<UiMountedHitTestMechanic>, UiMountedProjectionDenial> {
        semantic
            .order
            .iter()
            .filter_map(|instance| self.hit_tests.get(instance).copied())
            .filter(|row| row.surface() == surface && row.binding() == binding)
            .map(|row| {
                crate::mounting::projection::hit_test::reattribute_hit_test(row, frame, receipts)
            })
            .collect()
    }

    pub(in crate::mounting::projection::frame_storage) fn hit_test_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        frame: UiMountedFrameIdentity,
        receipts: &UiMountedNodeReceiptBasis,
    ) -> Result<Option<UiMountedHitTestMechanic>, UiMountedProjectionDenial> {
        self.hit_tests
            .get(&instance)
            .copied()
            .filter(|row| row.surface() == surface && row.binding() == binding)
            .map(|row| {
                crate::mounting::projection::hit_test::reattribute_hit_test(row, frame, receipts)
            })
            .transpose()
    }
}

fn receipt_for(
    receipts: &UiMountedNodeReceiptBasis,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    missing: UiMountedProjectionDenial,
) -> Result<worth_ui_host_contract::UiMountedNodeReceiptIdentity, UiMountedProjectionDenial> {
    receipts.receipt_for(instance).ok_or(missing)
}
