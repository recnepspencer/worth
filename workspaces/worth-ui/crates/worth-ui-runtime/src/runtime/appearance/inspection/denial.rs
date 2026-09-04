use super::producer::UiAppearanceInspectionProducer;

pub(super) fn record_attempt_denial(
    producer: &mut UiAppearanceInspectionProducer,
    context: &super::super::projection::UiAppearanceAttemptContext,
    denial: super::UiAppearanceInspectionDenial,
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
    let distinctions = receipt.change_distinctions();
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
            distinctions,
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
                context.theme_slots_compared(),
                context.consumers_selected(),
            ),
        )
        .with_denial_posture(denial_posture(denial));
        producer.record(query, explanation);
    }
}

fn denial_posture(
    denial: super::UiAppearanceInspectionDenial,
) -> worth_ui_inspection::UiAppearanceInspectionDenialPosture {
    match denial {
        super::UiAppearanceInspectionDenial::Basis => {
            worth_ui_inspection::UiAppearanceInspectionDenialPosture::Basis
        }
        super::UiAppearanceInspectionDenial::Resolution => {
            worth_ui_inspection::UiAppearanceInspectionDenialPosture::Resolution
        }
        super::UiAppearanceInspectionDenial::MountAffinity => {
            worth_ui_inspection::UiAppearanceInspectionDenialPosture::MountAffinity
        }
        super::UiAppearanceInspectionDenial::MountLowering => {
            worth_ui_inspection::UiAppearanceInspectionDenialPosture::MountLowering
        }
    }
}

#[cfg(test)]
mod tests {
    use super::denial_posture;

    #[test]
    fn maps_every_runtime_denial_to_its_public_posture() {
        use crate::runtime::appearance::inspection::UiAppearanceInspectionDenial as Denial;
        use worth_ui_inspection::UiAppearanceInspectionDenialPosture as Posture;

        assert_eq!(denial_posture(Denial::Basis), Posture::Basis);
        assert_eq!(denial_posture(Denial::Resolution), Posture::Resolution);
        assert_eq!(
            denial_posture(Denial::MountAffinity),
            Posture::MountAffinity
        );
        assert_eq!(
            denial_posture(Denial::MountLowering),
            Posture::MountLowering
        );
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
