use worth_ui_inspection::{
    UiAppearanceInspectionCost, UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
    UiAppearanceInspectionExplanation, UiAppearanceInspectionMountedMechanic,
    UiAppearanceInspectionPhysicalSuppression, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSourceSpan, UiAppearanceInspectionSupport, UiAppearanceInspectionValue,
};

pub(super) fn record_projection(
    producer: &mut super::producer::UiAppearanceInspectionProducer,
    scope: &super::producer::UiAppearanceInspectionScope,
    projection: &super::super::projection::UiAppearanceProjection,
    consumers_selected: u32,
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) {
    let distinctions = receipt.change_distinctions();
    let world = producer.current_world(projection.target().surface().diagnostic_value());
    for aspect in projection.aspects() {
        let query = UiAppearanceInspectionQuery::new(
            world,
            projection.target().graph_node().digest(),
            aspect.aspect(),
        );
        let explanation = UiAppearanceInspectionExplanation::new(
            query,
            projection.role().as_str(),
            projection.role_revision().value(),
            projection.theme(),
            projection.theme_revision(),
            aspect.state_classes().to_vec().into_boxed_slice(),
            UiAppearanceInspectionDecisionCell::new(
                aspect.decision_cell_ordinal(),
                aspect.state_classes().to_vec().into_boxed_slice(),
            ),
            UiAppearanceInspectionSourceSpan::Unavailable,
            value_source(aspect.provenance()),
            support(aspect.support()),
            UiAppearanceInspectionValue::Resolved(aspect.value()),
            distinctions,
            mounted_mechanic(receipt),
            physical_suppression(receipt),
            aspect.semantic_digest(),
            UiAppearanceInspectionEvidence::new(
                projection.state().basis().source_basis(),
                projection.state().basis().turn().as_u64(),
                *projection.state().basis().owner_revisions(),
            ),
            UiAppearanceInspectionCost::new(
                projection.state().classes().count() as u8,
                1,
                aspect.decision_cells_visited(),
                aspect.theme_slots_compared(),
                consumers_selected,
            ),
        );
        producer.record_scoped(scope, query, explanation);
    }
}

fn mounted_mechanic(
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) -> UiAppearanceInspectionMountedMechanic {
    if !receipt.mounting_result_available() {
        UiAppearanceInspectionMountedMechanic::NotAttempted
    } else if receipt.mounted_mechanical_output_changed() {
        UiAppearanceInspectionMountedMechanic::Changed
    } else {
        UiAppearanceInspectionMountedMechanic::Unchanged
    }
}

fn physical_suppression(
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) -> UiAppearanceInspectionPhysicalSuppression {
    if !receipt.mounting_result_available() {
        UiAppearanceInspectionPhysicalSuppression::NotAttempted
    } else if receipt.equal_output_suppressed() {
        UiAppearanceInspectionPhysicalSuppression::Suppressed
    } else {
        UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    }
}

fn support(
    support: super::super::projection::UiAppearanceSupportPosture,
) -> UiAppearanceInspectionSupport {
    match support {
        super::super::projection::UiAppearanceSupportPosture::Supported => {
            UiAppearanceInspectionSupport::Supported
        }
        super::super::projection::UiAppearanceSupportPosture::Unsupported => {
            UiAppearanceInspectionSupport::Unsupported
        }
        super::super::projection::UiAppearanceSupportPosture::Inapplicable => {
            UiAppearanceInspectionSupport::Inapplicable
        }
    }
}

fn value_source(
    provenance: &super::super::projection::UiAppearanceProvenance,
) -> worth_ui_inspection::UiAppearanceInspectionValueSource {
    use super::super::projection::UiAppearanceProvenance;
    use worth_ui_inspection::UiAppearanceInspectionValueSource;
    match provenance {
        UiAppearanceProvenance::Literal => UiAppearanceInspectionValueSource::Literal,
        UiAppearanceProvenance::ThemeSlot {
            selected_slot,
            terminal_slot,
            ..
        } => UiAppearanceInspectionValueSource::ThemeSlot {
            selected: selected_slot.as_str().into(),
            terminal: terminal_slot.as_str().into(),
        },
    }
}
