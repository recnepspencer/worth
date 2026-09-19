//! Query-free runtime derivation of presentation correspondence inputs.

use worth_ui_query_binding::{
    WorthUiPresentationMechanicBasisInput, WorthUiPresentationPaintSpanBasis,
    WorthUiPresentationPinBasis, WorthUiPresentationRequestBasis,
    WorthUiPresentationRequestBasisDenial, WorthUiPresentationRequestBasisInput,
};

use super::{preparation::mounted_semantic_text, UiNativeTextPresentationPrepared};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeTextPresentationCorrespondenceDenial {
    MissingHostLineage,
    PreparedMechanicMismatch,
    RequestBasis(WorthUiPresentationRequestBasisDenial),
}

pub(crate) fn derive_text_presentation_request_bases(
    consumption: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    prepared: &UiNativeTextPresentationPrepared,
    pins: worth_ui_host_contract::UiGlyphRasterPinTransitionView<'_>,
    binding_pins: &[worth_ui_host_contract::UiGlyphRasterPinRequest],
) -> Result<Box<[WorthUiPresentationRequestBasis]>, UiNativeTextPresentationCorrespondenceDenial> {
    let host_lineage = consumption
        .host_presentation_lineage()
        .ok_or(UiNativeTextPresentationCorrespondenceDenial::MissingHostLineage)?;
    let work = consumption.presentation_work();
    let affinity = work.affinity();
    let requirement = consumption.requirement();
    let semantic_work = mounted_semantic_text(work);
    let complete = semantic_work.complete;
    let mechanics = semantic_work.mechanics;
    if mechanics.len() != prepared.demand_batches().len() {
        return Err(UiNativeTextPresentationCorrespondenceDenial::PreparedMechanicMismatch);
    }
    let mechanic_bases = mechanics
        .into_iter()
        .zip(prepared.demand_batches())
        .map(|((command, mechanic), demand)| {
            if command.mounted_instance() != mechanic.mounted_instance()
                || demand.layout_identity() != mechanic.qualified_layout_identity()
                || demand.scale().text_scale_generation() != mechanic.qualified_layout_scale()
                || demand.scale().dpi_milli() != requirement.device_scale_milli()
            {
                return Err(UiNativeTextPresentationCorrespondenceDenial::PreparedMechanicMismatch);
            }
            Ok(WorthUiPresentationMechanicBasisInput {
                mounted_instance: mechanic.mounted_instance(),
                mechanic: command,
                content_generation: mechanic.content_generation(),
                content: std::sync::Arc::from(mechanic.text()),
                layout: demand.layout_identity(),
                layout_request: mechanic.qualified_layout_request(),
                layout_width: mechanic.qualified_layout_width(),
                paint_spans: mechanic
                    .foregrounds()
                    .iter()
                    .copied()
                    .map(WorthUiPresentationPaintSpanBasis::from_mounted)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                raster_keys: demand
                    .records()
                    .iter()
                    .map(|record| record.key())
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                text_scale: demand.scale().text_scale_generation(),
            })
        })
        .collect::<Result<Vec<_>, UiNativeTextPresentationCorrespondenceDenial>>()?;
    let basis = WorthUiPresentationRequestBasis::from_runtime_correspondence(
        WorthUiPresentationRequestBasisInput {
            mounted_frame: affinity.successor(),
            semantic_surface: affinity.surface(),
            host_surface: requirement.host_surface(),
            binding: affinity.binding(),
            complete,
            mechanics: mechanic_bases.into_boxed_slice(),
            removed_mechanics: semantic_work.removals.into_boxed_slice(),
            binding_pins: binding_pins
                .iter()
                .copied()
                .map(WorthUiPresentationPinBasis::from_runtime)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            pin_additions: pins
                .additions()
                .iter()
                .copied()
                .map(WorthUiPresentationPinBasis::from_runtime)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            pin_releases: pins
                .releases()
                .iter()
                .copied()
                .map(WorthUiPresentationPinBasis::from_runtime)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            dpi_milli: requirement.device_scale_milli(),
            attempt: consumption.attempt(),
            predecessor: affinity.predecessor(),
            host_lineage,
        },
    )
    .map_err(UiNativeTextPresentationCorrespondenceDenial::RequestBasis)?;
    Ok(vec![basis].into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, rc::Rc};

    use super::*;
    use crate::certification_support::{
        initial_presentation_mechanics_for_certification,
        semantic_text_projection_for_certification, UiSemanticTextProjectionCertificationMutation,
    };
    use crate::mounting::qualified_text_test_support::inert_qualified_layout;
    use crate::native_platform::text_presentation::{
        prepare_mounted_semantic_text, UiMountedEventTimeDpiAuthority,
        UiNativeTextPresentationPreparation,
    };
    use worth_ui_host_contract::{
        UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfaceIdentity,
        UiHostSurfacePresentationMode, UiMountedFrameConsumptionInput,
        UiMountedFrameConsumptionView, UiMountedPresentationAttemptIdentity,
        UiMountedPresentationWorkView, UiMountedSurfaceBindingRequirement, UiPresentationDeadline,
        WorthUiHostCapabilityObservationGeneration,
    };

    #[test]
    fn prepared_runtime_work_carries_the_exact_typed_presentation_basis() {
        let projection = semantic_text_projection_for_certification(
            UiSemanticTextProjectionCertificationMutation::Exact,
        );
        let requirement = requirement(&projection);
        let mechanics = initial_presentation_mechanics_for_certification(&projection, requirement);
        let layout = inert_qualified_layout("ONLINE");
        let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
        let Some(UiNativeTextPresentationPreparation::Prepared(prepared)) =
            prepare_mounted_semantic_text(
                UiMountedPresentationWorkView::Initial(&mechanics),
                dpi,
                |identity| (identity == layout.identity()).then_some(layout.as_ref()),
            )
        else {
            panic!("exact mounted semantic text must prepare");
        };
        let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let view = consumption_view(&mechanics, requirement, attempt);

        let bases = derive_text_presentation_request_bases(
            &view,
            &prepared,
            worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
            &[],
        )
        .unwrap();
        let basis = bases.first().expect("the one text mechanic has one basis");
        let demand = &prepared.demand_batches()[0];
        let (command, mechanic) =
            mounted_semantic_text(UiMountedPresentationWorkView::Initial(&mechanics)).mechanics[0];

        assert_eq!(bases.len(), 1);
        let mechanic_basis = basis.mechanics().first().unwrap();
        assert_eq!(mechanic_basis.mechanic(), command);
        assert_eq!(
            mechanic_basis.mounted_instance(),
            mechanic.mounted_instance()
        );
        assert_eq!(mechanic_basis.layout(), demand.layout_identity());
        let expected_raster_keys = demand
            .records()
            .iter()
            .map(|record| record.key())
            .collect::<HashSet<_>>();
        let admitted_raster_keys = mechanic_basis
            .raster_key_set()
            .keys()
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        assert_eq!(admitted_raster_keys, expected_raster_keys);
        assert_eq!(basis.dpi_milli(), requirement.device_scale_milli());
        assert_eq!(
            mechanic_basis.text_scale(),
            mechanic.qualified_layout_scale()
        );
        assert_eq!(basis.attempt(), attempt);
        assert_eq!(basis.semantic_surface(), requirement.semantic_surface());
        assert_eq!(basis.host_surface(), requirement.host_surface());
        assert_eq!(basis.binding(), requirement.binding());
        assert_eq!(
            mechanic_basis.paint_spans().len(),
            mechanic.foregrounds().len()
        );
        assert_eq!(
            basis.host_lineage(),
            view.host_presentation_lineage().unwrap()
        );
    }

    fn requirement(
        projection: &worth_ui_host_contract::UiMountedProjectionView,
    ) -> UiMountedSurfaceBindingRequirement {
        UiMountedSurfaceBindingRequirement::new(
            projection.surface(),
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            projection.binding(),
            WorthUiHostCapabilityObservationGeneration::new(7),
            11,
            UiHostSurfacePresentationMode::NativeDisplay,
        )
    }

    fn consumption_view<'a>(
        mechanics: &'a worth_ui_host_contract::UiMountedPresentationInitial,
        requirement: UiMountedSurfaceBindingRequirement,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> UiMountedFrameConsumptionView<'a> {
        let UiHostProtocolNegotiation::Compatible(protocol) =
            UiHostProtocolContract::current().negotiate()
        else {
            panic!("the current host protocol must negotiate");
        };
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: Rc::new(()),
            host_session_identity: 41,
            protocol,
            capability_generation: requirement.capability_generation(),
            capability_profile_digest: requirement.capability_profile_digest(),
            attempt,
            deadline: UiPresentationDeadline::at_tick(100),
            requirement,
            presentation_work: UiMountedPresentationWorkView::Initial(mechanics),
            appearance_work: None,
            qualified_text: &(),
            text_raster_work: None,
        })
    }
}
