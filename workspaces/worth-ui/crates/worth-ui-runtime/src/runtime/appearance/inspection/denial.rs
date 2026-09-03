use super::producer::UiAppearanceInspectionProducer;

pub(super) fn record_attempt_denial(
    producer: &mut UiAppearanceInspectionProducer,
    context: &super::super::projection::UiAppearanceAttemptContext,
    _denial: super::UiAppearanceInspectionDenial,
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) {
    let world = world_for_context(context);
    let role = context.role();
    let role_name = role.map_or("unavailable", |role| role.role().as_str());
    let role_revision = role.map_or(0, |role| role.revision().value());
    let theme = context.theme_identity().unwrap_or("unavailable");
    let state_classes = context
        .state()
        .map(|state| state.classes().map(|(_, class)| class).collect::<Vec<_>>())
        .unwrap_or_default();
    let owner_revisions = context
        .state()
        .map_or([0; 6], |state| *state.basis().owner_revisions());
    let (source_basis, turn) = context
        .state()
        .map_or((context.owner_evidence(), 0), |state| {
            (state.basis().source_basis(), state.basis().turn().as_u64())
        });
    let aspects = if context.aspect_hints().is_empty() {
        context
            .aspects()
            .iter()
            .map(|aspect| aspect.aspect())
            .collect()
    } else {
        context.aspect_hints().to_vec()
    };
    for aspect in aspects {
        let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
            world,
            context.target().graph_node().digest(),
            aspect,
        );
        let explanation = worth_ui_inspection::UiAppearanceInspectionExplanation::new(
            query,
            role_name,
            role_revision,
            theme,
            context.theme_revision(),
            state_classes.clone().into_boxed_slice(),
            worth_ui_inspection::UiAppearanceInspectionDecisionCell::new(
                0,
                state_classes.clone().into_boxed_slice(),
            ),
            worth_ui_inspection::UiAppearanceInspectionSourceSpan::Unavailable,
            "unresolved",
            "unresolved",
            worth_ui_inspection::UiAppearanceInspectionSupport::Supported,
            worth_ui_inspection::UiAppearanceInspectionValue::Missing,
            super::producer::invalidation_cause(receipt),
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::NotAttempted,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotAttempted,
            context.owner_evidence(),
            worth_ui_inspection::UiAppearanceInspectionEvidence::new(
                source_basis,
                turn,
                owner_revisions,
            ),
            worth_ui_inspection::UiAppearanceInspectionCost::new(
                state_classes.len() as u8,
                0,
                0,
                context.catalog_revision() as u32,
                context.consumers_selected(),
            ),
        );
        producer.record(query, explanation);
    }
}

fn world_for_context(
    context: &super::super::projection::UiAppearanceAttemptContext,
) -> worth_ui_inspection::UiAppearanceInspectionWorld {
    worth_ui_inspection::UiAppearanceInspectionWorld::new(
        context.target().session().as_u64(),
        context
            .generation()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        context.target().surface().diagnostic_value(),
    )
}
