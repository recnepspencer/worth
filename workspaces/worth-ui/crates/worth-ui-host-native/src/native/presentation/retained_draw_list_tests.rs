use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiMountedAllocationBasis, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedClipTable, UiMountedContentGeneration, UiMountedCoordinateSpace,
    UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView, UiMountedFrameIdentity,
    UiMountedHitTestProjection, UiMountedHitTestTable, UiMountedInstanceIdentity,
    UiMountedLayerTable, UiMountedLogicalDamage, UiMountedMechanicalRole,
    UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput, UiMountedNodeReceiptIssuer,
    UiMountedOmissionReason, UiMountedPaintBatchTable, UiMountedPaintCommand,
    UiMountedPaintCommandChange, UiMountedPaintOrderEdit, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPaintProjection, UiMountedParticipation,
    UiMountedParticipationFact, UiMountedParticipationInput, UiMountedParticipationStatus,
    UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionInput,
    UiMountedPortalOverlayLifecyclePosture, UiMountedPortalOverlayMechanic,
    UiMountedPortalOverlayReference, UiMountedPortalOverlayTable,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationDelta,
    UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationUnchanged,
    UiMountedPresentationUnchangedInput, UiMountedPresentationWorkView, UiMountedProjectionView,
    UiMountedProjectionViewInput, UiMountedRealtimeBatchTable, UiMountedResourceTable,
    UiMountedRgba8, UiMountedSemanticTextTable, UiMountedSpatialBatchTable,
    UiMountedSurfaceBindingRequirement, UiMountedTransformProjection, UiPresentationDeadline,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial};

#[path = "retained_draw_list/delta_transaction_tests.rs"]
mod delta_transaction_tests;
#[path = "retained_draw_list/reconstruction_tests.rs"]
mod reconstruction_tests;
#[path = "retained_draw_list/replay_tests.rs"]
mod replay_tests;
#[path = "retained_draw_list/superseded_transaction_tests.rs"]
mod superseded_transaction_tests;

#[test]
fn unchanged_advances_exact_affinity_without_draw_order_or_damage_work() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial = world.initial(
        frame,
        [world.rect(frame, world.first, 0.0, UiMountedRgba8::new(1, 2, 3, 255))],
    );
    let identity = initial.commands()[0].identity();
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let successor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_receipts = UiMountedNodeReceiptIssuer::mint_for(successor).unwrap();
    let unchanged =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: frame,
            successor,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            production_cost: Default::default(),
        })
        .with_successor_receipt_affinity(Some(successor_receipts.receipt_affinity()));
    retained.apply_unchanged(&unchanged).unwrap();
    assert_eq!(retained.frame, successor);
    assert_eq!(retained.command(identity), Some(&initial.commands()[0]));
    assert_eq!(retained.order.ordered().count(), 1);
    assert_eq!(
        retained.realized_regions().unwrap()[0].mounted_receipt(),
        successor_receipts.receipt_for(world.first)
    );
    assert_eq!(
        retained.top_paint_attribution().unwrap().1.node_receipt,
        successor_receipts.receipt_for(world.first)
    );
}

#[test]
fn stale_delta_denies_without_mutating_retained_commands() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mechanic = world.rect(frame, world.first, 0.0, UiMountedRgba8::new(1, 2, 3, 255));
    let initial = world.initial(frame, [mechanic]);
    let identity = initial.commands()[0].identity();
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: Vec::new(),
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: initial.order_integrity(),
        damage: Vec::new(),
        auxiliary: None,
        production_cost: Default::default(),
    });
    assert!(matches!(
        retained.apply_delta(&delta),
        Err(UiNativeRetainedDrawListDenial::AffinityMismatch)
    ));
    assert_eq!(retained.command(identity), Some(&initial.commands()[0]));
}

pub(in crate::native) struct DrawListWorld {
    pub(in crate::native) surface: UiSemanticSurfaceIdentity,
    pub(in crate::native) binding: UiSurfaceBindingGeneration,
    pub(in crate::native) content: UiMountedContentGeneration,
    pub(in crate::native) first: UiMountedInstanceIdentity,
    second: UiMountedInstanceIdentity,
    pub(in crate::native::presentation) third: UiMountedInstanceIdentity,
    pub(in crate::native) requirement: UiMountedSurfaceBindingRequirement,
}

impl DrawListWorld {
    pub(in crate::native) fn new() -> Self {
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let requirement = UiMountedSurfaceBindingRequirement::new(
            surface,
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            binding,
            WorthUiHostCapabilityObservationGeneration::new(7),
            11,
            UiHostSurfacePresentationMode::NativeDisplay,
        );
        Self {
            surface,
            binding,
            content: UiMountedContentGeneration::mint_unbound().unwrap(),
            first: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            second: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            third: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            requirement,
        }
    }

    pub(in crate::native::presentation) fn rect(
        &self,
        frame: UiMountedFrameIdentity,
        instance: UiMountedInstanceIdentity,
        x: f32,
        color: UiMountedRgba8,
    ) -> UiMountedPortalOverlayMechanic {
        self.rect_at_order(frame, instance, x, color, 0)
    }

    pub(in crate::native::presentation) fn rect_at_order(
        &self,
        frame: UiMountedFrameIdentity,
        instance: UiMountedInstanceIdentity,
        x: f32,
        color: UiMountedRgba8,
        layer_semantic_order: u32,
    ) -> UiMountedPortalOverlayMechanic {
        let bounds = canonical_box(x, 0.0, 32.0, 24.0);
        UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame,
                surface: self.surface,
                binding: self.binding,
                owner: instance,
                owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(instance),
                portal_identity: instance.diagnostic_value(),
                anchor_presentation: UiHostObservationPresentationBasis::new(
                    self.requirement.host_surface(),
                    frame,
                    self.binding,
                    UiHostPresentationEpoch::issued_by_host(1),
                ),
                anchor_bounds: bounds,
                bounds,
                paint_bounds: bounds,
                color,
                layer_semantic_order,
                layer_depth: 0,
                clip_bounds: bounds,
                lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
                shielding: UiMountedPortalInputShielding::ContentBounds,
            },
        )
        .unwrap()
    }

    pub(in crate::native) fn initial<const N: usize>(
        &self,
        frame: UiMountedFrameIdentity,
        rows: [UiMountedPortalOverlayMechanic; N],
    ) -> UiMountedPresentationInitial {
        let commands = rows
            .iter()
            .map(|mechanic| command(*mechanic))
            .collect::<Vec<_>>();
        let order = commands
            .iter()
            .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
            .collect::<Vec<_>>();
        let projection = projection(self, frame, rows.to_vec());
        UiMountedPresentationInitial::from_inert_mechanics(UiMountedPresentationInitialInput {
            successor: frame,
            surface: self.surface,
            binding: self.binding,
            content: self.content,
            baseline: self.requirement.baseline(),
            projection,
            commands,
            order_integrity: UiMountedPaintOrderIntegrity::for_order(&order),
            order,
            damage: rows
                .iter()
                .map(|row| UiMountedLogicalDamage::from_runtime_mounting(row.bounds()))
                .collect(),
            production_cost: Default::default(),
        })
    }

    /// Runtime-issued work for this world's surface, as a host consumes it.
    pub(in crate::native) fn consumption_view<'work>(
        &self,
        initial: &'work UiMountedPresentationInitial,
    ) -> UiMountedFrameConsumptionView<'work> {
        let UiHostProtocolNegotiation::Compatible(protocol) =
            UiHostProtocolContract::current().negotiate()
        else {
            panic!("current protocol must negotiate");
        };
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: std::rc::Rc::new(()),
            host_session_identity: 1,
            protocol,
            capability_generation: self.requirement.capability_generation(),
            capability_profile_digest: self.requirement.capability_profile_digest(),
            attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            deadline: UiPresentationDeadline::at_tick(20),
            requirement: self.requirement,
            presentation_work: UiMountedPresentationWorkView::Initial(initial),
            appearance_work: None,
            qualified_text: &(),
            text_raster_work: None,
        })
    }

    pub(in crate::native) fn retained_focus_target(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> (
        super::UiNativeRetainedDrawList,
        worth_ui_host_contract::UiHostFocusPlacementTarget,
    ) {
        let row = self.rect(frame, self.first, 0.0, UiMountedRgba8::new(9, 17, 31, 255));
        let target = worth_ui_host_contract::UiHostFocusPlacementTarget::new(
            row.owner(),
            row.owner_receipt(),
        );
        (
            super::UiNativeRetainedDrawList::initial(&self.initial(frame, [row]), &[]).unwrap(),
            target,
        )
    }
}

pub(in crate::native::presentation) fn command(
    mechanic: UiMountedPortalOverlayMechanic,
) -> UiMountedPaintCommand {
    UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
        mechanic,
    }
}

fn projection(
    world: &DrawListWorld,
    frame: UiMountedFrameIdentity,
    rows: Vec<UiMountedPortalOverlayMechanic>,
) -> UiMountedProjectionView {
    let nodes = rows
        .iter()
        .enumerate()
        .map(|(index, row)| rect_node(index, row))
        .collect();
    let authored_paint_commands = rows.iter().copied().map(command).collect::<Vec<_>>();
    let mut authored_paint_order = authored_paint_commands
        .iter()
        .enumerate()
        .map(|(ordinal, command)| {
            (
                command.layer_semantic_order(),
                ordinal,
                UiMountedPaintOrderIdentity::for_command(command.identity()),
            )
        })
        .collect::<Vec<_>>();
    authored_paint_order.sort_by_key(|source| (source.0, source.1));
    let authored_paint_order = authored_paint_order
        .into_iter()
        .map(|source| source.2)
        .collect();
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame,
        surface: world.surface,
        binding: world.binding,
        content_generation: world.content,
        nodes,
        clips: UiMountedClipTable::produced(Vec::new()),
        layers: UiMountedLayerTable::produced(Vec::new()),
        portal_overlays: UiMountedPortalOverlayTable::from_runtime_mounting(rows).unwrap(),
        semantic_text: UiMountedSemanticTextTable::empty(),
        hit_tests: UiMountedHitTestTable::empty(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands,
        authored_paint_order,
    })
}

fn rect_node(index: usize, row: &UiMountedPortalOverlayMechanic) -> UiMountedNodeProjectionView {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
    let reference = UiMountedPortalOverlayReference::from_runtime_mounting(index as u16);
    UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
        mounted_instance: row.owner(),
        node_receipt: row.owner_receipt(),
        authored_position: u64::try_from(index).expect("fixture authored position"),
        role: UiMountedMechanicalRole::Control,
        participation: UiMountedParticipation::new(UiMountedParticipationInput {
            paint: admitted,
            clip: admitted,
            input: withheld,
            focus: withheld,
            hit_test: withheld,
            accessibility: withheld,
            motion: withheld,
            diagnostic: withheld,
        }),
        allocation: worth_ui_host_contract::UiMountedAllocationProjection::Known {
            bounds: row.bounds(),
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        },
        preview: worth_ui_host_contract::UiMountedPreviewProjection::Omitted(omitted),
        paint: UiMountedPaintProjection::Omitted(omitted),
        hit_test: UiMountedHitTestProjection::Omitted(omitted),
        accessibility: worth_ui_host_contract::UiMountedAccessibilityProjection::Omitted(omitted),
        motion: worth_ui_host_contract::UiMountedMotionProjection::Omitted(omitted),
        diagnostic: worth_ui_host_contract::UiMountedDiagnosticProjection::Omitted(omitted),
        drawables: vec![
            worth_ui_host_contract::UiMountedDrawableReference::PortalOverlay(reference),
        ],
        semantic_text: Vec::new(),
        portal_presentation: None,
    })
}

fn canonical_box(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}
