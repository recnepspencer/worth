//! Sealed mounted facts and real qualified text for the native coverage boundary.
//! No application-world or physical presentation acceptance is asserted here.

mod image_oracle;
mod token_suffix;

use super::{
    prepare_complete_semantic_text, UiMountedEventTimeDpiAuthority, UiNativeTextAtlasTransaction,
    UiNativeTextPresentationPreparation,
};
use std::{rc::Rc, sync::Arc};
use worth_ui_host_contract::*;

pub(super) struct CoverageWorld {
    pub(super) layout: Arc<worth_ui_text::UiQualifiedTextLayout>,
    pub(super) fragment: UiUnpublishedAppearanceFragment,
    pub(super) foreground: UiMountedTextForegroundAppearanceMechanic,
    pub(super) foregrounds: Box<[UiMountedTextForegroundAppearanceMechanic]>,
    presentation: UiMountedPresentationUnchanged,
    pub(super) attempt: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
}

impl CoverageWorld {
    pub(super) fn new(
        source: &str,
        instance: UiMountedInstanceIdentity,
        x: f32,
        clip: [f32; 4],
    ) -> Self {
        Self::in_binding(source, instance, x, clip, None)
    }

    pub(super) fn in_binding(
        source: &str,
        instance: UiMountedInstanceIdentity,
        x: f32,
        clip: [f32; 4],
        basis: Option<&Self>,
    ) -> Self {
        Self::with_commands(
            source,
            instance,
            x,
            clip,
            basis,
            &[(UiSemanticTextSlot::Value, 0.0)],
        )
    }

    pub(super) fn with_commands(
        source: &str,
        instance: UiMountedInstanceIdentity,
        x: f32,
        clip: [f32; 4],
        basis: Option<&Self>,
        commands: &[(UiSemanticTextSlot, f32)],
    ) -> Self {
        Self::with_paint(
            source,
            instance,
            x,
            clip,
            basis,
            commands,
            ([180, 30, 60, 255], u16::MAX),
        )
    }

    pub(super) fn with_paint(
        source: &str,
        instance: UiMountedInstanceIdentity,
        x: f32,
        clip: [f32; 4],
        basis: Option<&Self>,
        commands: &[(UiSemanticTextSlot, f32)],
        paint: ([u8; 4], u16),
    ) -> Self {
        Self::with_physical_geometry(
            source,
            instance,
            x,
            clip,
            basis,
            commands,
            paint,
            (160.0, 1_000),
        )
    }

    pub(super) fn with_physical_geometry(
        source: &str,
        instance: UiMountedInstanceIdentity,
        x: f32,
        clip: [f32; 4],
        basis: Option<&Self>,
        commands: &[(UiSemanticTextSlot, f32)],
        paint: ([u8; 4], u16),
        physical: (f32, u32),
    ) -> Self {
        let layout = crate::mounting::qualified_text_test_support::inert_qualified_layout(source);
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let predecessor = basis.map_or_else(
            || UiMountedFrameIdentity::mint_unbound().unwrap(),
            |world| world.fragment.work().successor().frame(),
        );
        let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let receipt = issuer.receipt_for(instance);
        let surface = basis.map_or_else(
            || UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            |world| world.requirement.semantic_surface(),
        );
        let binding = basis.map_or_else(
            || UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            |world| world.requirement.binding(),
        );
        let content = UiMountedContentGeneration::mint_unbound().unwrap();
        let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let requirement = UiMountedSurfaceBindingRequirement::new(
            surface,
            basis.map_or_else(
                || UiHostSurfaceIdentity::mint_unbound().unwrap(),
                |world| world.requirement.host_surface(),
            ),
            binding,
            WorthUiHostCapabilityObservationGeneration::new(7),
            11,
            UiHostSurfacePresentationMode::RecordOnly,
        );
        let requirement = UiMountedSurfaceBindingRequirement::with_baseline_and_device_scale(
            requirement.semantic_surface(),
            requirement.host_surface(),
            requirement.binding(),
            requirement.capability_generation(),
            requirement.capability_profile_digest(),
            requirement.presentation_mode(),
            requirement.baseline(),
            physical.1,
        );
        let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([17; 32]);
        let candidates = commands
            .iter()
            .map(|&(slot, offset)| {
                let x = x + offset;
                UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
                    UiMountedSemanticTextCompletionInput {
                        content_generation: content,
                        frame,
                        surface,
                        binding,
                        mounted_instance: instance,
                        node_receipt: receipt,
                        allocation_basis: UiMountedAllocationBasis::new(
                            1,
                            1,
                            1,
                            UiMountedTransformProjection::Identity,
                        ),
                        bounds: bounds([x, 0.0, physical.0, 48.0]),
                        clip_bounds: bounds(clip),
                        origin_x: x,
                        origin_y: 0.0,
                        text: Arc::from(source),
                        layout: layout.view(),
                        slot,
                        collection_row: None,
                        foregrounds: Arc::from([
                            UiMountedTextForegroundSpan::from_runtime_mounting(
                                UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
                                UiMountedRgba8::new(255, 255, 255, 255),
                                span,
                            ),
                        ]),
                        profile: UiSemanticTextProfile::BodyDefault,
                        layer_semantic_order: 1,
                        capability_generation: requirement.capability_generation(),
                        capability_profile_digest: requirement.capability_profile_digest(),
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let foregrounds = candidates
            .iter()
            .map(|candidate| {
                UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
                    UiMountedTextForegroundAppearanceCompletionInput {
                        issuer,
                        node_receipt: receipt,
                        command: UiMountedPaintCommandIdentity::semantic_text(candidate),
                        paint_span: span,
                        foreground: UiMountedAppearanceColor::from_straight_srgba(paint.0),
                        opacity: UiMountedPresentationOpacity::from_runtime_composition(paint.1),
                        projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                            issuer, 1, 1,
                        )
                        .unwrap(),
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let rows = foregrounds
            .iter()
            .cloned()
            .map(UiMountedAppearanceMechanic::TextForeground)
            .collect::<Vec<_>>();
        let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            surface,
            attempt,
            1,
            1,
            [],
        )
        .unwrap();
        let appearance = UiMountedAppearanceFrame::from_runtime_mounting(
            frame,
            surface,
            rows.iter().cloned(),
            order,
        )
        .unwrap();
        let previous_receipt = basis
            .filter(|world| world.foreground.node_receipt().mounted_instance() == instance)
            .map(|world| world.foreground.node_receipt());
        let previous_identities = basis
            .map(|world| {
                world
                    .foregrounds
                    .iter()
                    .filter(|value| value.node_receipt().mounted_instance() == instance)
                    .map(|value| {
                        UiMountedAppearanceMechanic::TextForeground(value.clone()).identity()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let successor_identities = rows
            .iter()
            .map(UiMountedAppearanceMechanic::identity)
            .collect::<std::collections::HashSet<_>>();
        let previous_identity_set = previous_identities
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut changes = rows
            .iter()
            .cloned()
            .map(|row| {
                let identity = row.identity();
                if previous_identity_set.contains(&identity) {
                    UiMountedAppearanceMechanicChange::replacement(identity, row)
                        .expect("matching exact appearance identity replaces")
                } else {
                    UiMountedAppearanceMechanicChange::Insert(row)
                }
            })
            .collect::<Vec<_>>();
        changes.extend(
            previous_identities
                .iter()
                .filter(|identity| !successor_identities.contains(*identity))
                .cloned()
                .map(UiMountedAppearanceMechanicChange::Remove),
        );
        let work = UiMountedAppearanceWork::from_runtime_mounting(
            UiMountedAppearanceWorkPosture::Delta,
            Some(predecessor),
            Some(
                UiMountedAppearancePredecessorManifest::from_runtime_mounting(
                    previous_identities,
                    [],
                )
                .unwrap(),
            ),
            appearance,
            changes,
            [],
            false,
        )
        .unwrap();
        let presentation = UiMountedPresentationUnchanged::from_inert_mechanics(
            UiMountedPresentationUnchangedInput {
                predecessor,
                successor: frame,
                surface,
                binding,
                content,
                baseline: requirement.baseline(),
                production_cost: Default::default(),
            },
        )
        .with_successor_receipt_affinity(Some(issuer.receipt_affinity()));
        let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: previous_receipt,
                successor: Some(receipt),
            },
            work,
            candidates,
            requirement,
            presentation.affinity(),
        )
        .unwrap();
        Self {
            layout,
            fragment,
            foreground: foregrounds[0].clone(),
            foregrounds: foregrounds.into_boxed_slice(),
            presentation,
            attempt,
            requirement,
        }
    }

    pub(super) fn with_native<Output>(
        &self,
        attempt: UiMountedPresentationAttemptIdentity,
        operation: impl FnOnce(&UiMountedFrameConsumptionView<'_>) -> Output,
    ) -> (Output, super::rasterization::UiNativeTextRasterWorkReport) {
        let UiNativeTextPresentationPreparation::Prepared(prepared) =
            prepare_complete_semantic_text(
                self.fragment.text_candidates(),
                UiMountedEventTimeDpiAuthority::from_requirement(self.requirement).unwrap(),
                UiGlyphRasterLane::Ordinary,
                |id| (id == self.layout.identity()).then_some(self.layout.as_ref()),
            )
            .unwrap()
        else {
            panic!("complete text preparation denied");
        };
        let mut cache = worth_ui_text::UiGlyphRasterCache::default();
        let mut transaction = UiNativeTextAtlasTransaction::prepare(
            &prepared,
            |id| (id == self.layout.identity()).then_some(self.layout.as_ref()),
            &mut cache,
        )
        .unwrap();
        let resolver = Resolver(&self.layout);
        transaction.with_mounted_work(
            UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
            &[],
            |raster| {
                let UiHostProtocolNegotiation::Compatible(protocol) =
                    UiHostProtocolContract::current().negotiate()
                else {
                    panic!("current protocol denied");
                };
                let view = UiMountedFrameConsumptionView::from_inert_mechanics(
                    UiMountedFrameConsumptionInput {
                        authority: Rc::new(()),
                        host_session_identity: 41,
                        protocol,
                        capability_generation: self.requirement.capability_generation(),
                        capability_profile_digest: self.requirement.capability_profile_digest(),
                        attempt,
                        deadline: UiPresentationDeadline::at_tick(100),
                        requirement: self.requirement,
                        presentation_work: UiMountedPresentationWorkView::Unchanged(
                            &self.presentation,
                        ),
                        qualified_text: &resolver,
                        text_raster_work: Some(raster),
                    },
                );
                operation(&view)
            },
        )
    }
}

struct Resolver<'a>(&'a worth_ui_text::UiQualifiedTextLayout);
impl UiMountedQualifiedTextResolver for Resolver<'_> {
    fn resolve(
        &self,
        identity: UiQualifiedTextLayoutIdentity,
    ) -> Option<UiQualifiedTextLayoutView<'_>> {
        (identity == self.0.identity()).then(|| self.0.view())
    }
}
fn bounds(value: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: value[0],
        y: value[1],
        width: value[2],
        height: value[3],
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
