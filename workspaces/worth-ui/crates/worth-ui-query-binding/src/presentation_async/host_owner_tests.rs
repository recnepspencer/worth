use super::*;
use crate::presentation_async::{
    WorthUiPresentationMechanicBasisInput, WorthUiPresentationPaintSpanBasis,
    WorthUiPresentationPinBasis, WorthUiPresentationRequestBasisInput,
};

#[path = "host_owner_tests/completion.rs"]
mod completion;
pub(super) use completion::native_paint_completion;

#[test]
fn host_owner_reuses_one_presealed_source_across_successive_presentations() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let graph = owner
        .workspace
        .owned_async_runtime_topology()
        .unwrap()
        .signal_graph_instance();

    let baseline = owner.admit_pending(sequence.baseline).unwrap();
    assert_eq!(baseline.observation().signal_graph_instance(), graph);
    owner
        .admit_presented(&baseline, &native_paint_completion(1))
        .unwrap();

    let successor = owner.admit_pending(sequence.successor).unwrap();
    assert_eq!(successor.observation().signal_graph_instance(), graph);
    let presented = owner
        .admit_presented(&successor, &native_paint_completion(2))
        .unwrap();
    assert_eq!(
        presented.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    let retired = owner.workspace.owned_async_runtime_topology().unwrap();
    assert_eq!(
        retired.installed_conditional_nodes(),
        0,
        "presentation async lifecycle must not install ambient conditional nodes"
    );
    assert_eq!(
        retired.installed_async_declarations(),
        1,
        "one presealed source declaration serves every presentation request"
    );
    assert_eq!(
        retired.active_signal_nodes(),
        retired.installed_conditional_nodes() + retired.installed_async_declarations(),
        "the retained graph must contain no dynamic or superseded source nodes"
    );
}

pub(super) struct PresentationSequence {
    pub(super) baseline: WorthUiPresentationRequestBasis,
    pub(super) successor: WorthUiPresentationRequestBasis,
}

pub(super) fn presentation_sequence() -> PresentationSequence {
    let semantic_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let host_surface = worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap();
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let lineage =
        worth_ui_host_contract::UiHostPresentationLineageIdentity::from_certification_host_session(
            71,
        )
        .unwrap();
    let baseline_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let layout =
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics([3; 32]);
    let pin = WorthUiPresentationPinBasis::from_runtime(
        worth_ui_host_contract::UiGlyphRasterPinRequest::from_text_mechanics(layout, raster_key()),
    );
    let baseline = basis_for_lineage(
        semantic_surface,
        host_surface,
        binding,
        lineage,
        baseline_frame,
        None,
        true,
        vec![mechanic(0, layout)].into_boxed_slice(),
        vec![pin].into_boxed_slice(),
        vec![pin].into_boxed_slice(),
        Box::new([]),
    );
    let successor = basis_for_lineage(
        semantic_surface,
        host_surface,
        binding,
        lineage,
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        Some(baseline_frame),
        false,
        Box::new([]),
        Box::new([]),
        Box::new([]),
        vec![pin].into_boxed_slice(),
    );
    PresentationSequence {
        baseline,
        successor,
    }
}

pub(super) struct InstalledTestOwner {
    owner: WorthUiPresentationAsyncOwner,
    correspondence: WorthUiPresentationCorrespondenceIssuer,
}

impl InstalledTestOwner {
    pub(super) fn admit_pending(
        &mut self,
        basis: WorthUiPresentationRequestBasis,
    ) -> Result<WorthUiPresentationPendingReceipt, WorthUiPresentationPendingAdmissionDenial> {
        let correspondence = self.correspondence.issue(basis).unwrap();
        self.owner.admit_pending(correspondence)
    }

    pub(super) fn admit_presented(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
        completion: &worth_ui_host_contract::UiMountedSurfacePresentationCompletion,
    ) -> Result<WorthUiPresentationPresentedReceipt, WorthUiPresentationSettlementDenial> {
        let completion = self
            .correspondence
            .certify_presented(receipt, std::mem::size_of_val(completion) as u64);
        self.owner.admit_presented(receipt, completion)
    }

    pub(super) fn admit_effects_indeterminate(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
    ) -> Result<WorthUiPresentationUnresolvedReceipt, WorthUiPresentationSettlementDenial> {
        let observation = self
            .correspondence
            .certify_effects_indeterminate(receipt, 0);
        self.owner.admit_effects_indeterminate(receipt, observation)
    }

    pub(super) fn admit_superseded_physical(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
    ) -> Result<WorthUiPresentationPresentedReceipt, WorthUiPresentationSettlementDenial> {
        let observation = self.correspondence.certify_superseded_physical(receipt, 0);
        self.owner.admit_superseded_physical(receipt, observation)
    }

    pub(super) fn cancel_after_effects_may_have_begun(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
    ) -> Result<WorthUiPresentationUnresolvedReceipt, WorthUiPresentationSettlementDenial> {
        let observation = self
            .correspondence
            .certify_cancellation_effects_may_have_begun(receipt, 0);
        self.owner
            .cancel_after_effects_may_have_begun(receipt, observation)
    }

    pub(super) fn admit_effects_indeterminate_requiring_reconstruction(
        &mut self,
        receipt: &WorthUiPresentationPendingReceipt,
    ) -> Result<WorthUiPresentationRecoveryRequiredReceipt, WorthUiPresentationSettlementDenial>
    {
        let observation = self
            .correspondence
            .certify_effects_indeterminate(receipt, 0);
        self.owner
            .admit_effects_indeterminate_requiring_reconstruction(receipt, observation)
    }
}

impl std::ops::Deref for InstalledTestOwner {
    type Target = WorthUiPresentationAsyncOwner;

    fn deref(&self) -> &Self::Target {
        &self.owner
    }
}

impl std::ops::DerefMut for InstalledTestOwner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.owner
    }
}

pub(super) fn installed_owner() -> InstalledTestOwner {
    let plan = WorthUiPresentationAsyncHostPlan::prepare().unwrap();
    let (request, completion) = plan.into_parts();
    let installation =
        worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(request.generation(), request.into_packages())
            .unwrap();
    let installation = completion.complete(installation).unwrap();
    let (owner, correspondence) = installation.into_runtime_parts();
    InstalledTestOwner {
        owner,
        correspondence,
    }
}

pub(super) fn basis(slot: u16) -> WorthUiPresentationRequestBasis {
    let mounted_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let paint = worth_ui_host_contract::UiMountedTextForegroundSpan::from_runtime_mounting(
        worth_ui_host_contract::UiTextOriginalRange::new(0, 4).unwrap(),
        worth_ui_host_contract::UiMountedRgba8::new(1, 2, 3, 255),
        worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]),
    );
    WorthUiPresentationRequestBasis::from_runtime_correspondence(
        WorthUiPresentationRequestBasisInput {
            mounted_frame: worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
            semantic_surface:
                worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            host_surface: worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
            binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            complete: true,
            mechanics: vec![WorthUiPresentationMechanicBasisInput {
                mounted_instance,
                mechanic: worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
                    mounted_instance,
                    slot,
                    None,
                ),
                content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound().unwrap(),
                content: std::sync::Arc::from("text"),
                layout: worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics([3; 32]),
                layout_request: worth_ui_host_contract::UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([2; 32]),
                layout_width: worth_ui_host_contract::UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
                paint_spans: vec![WorthUiPresentationPaintSpanBasis::from_mounted(paint)]
                    .into_boxed_slice(),
                raster_keys: vec![raster_key()].into_boxed_slice(),
                text_scale: worth_ui_host_contract::UiTextScaleGeneration::new(2).unwrap(),
            }]
            .into_boxed_slice(),
            removed_mechanics: Box::new([]),
            binding_pins: Box::new([]),
            pin_additions: Box::new([]),
            pin_releases: Box::new([]),
            dpi_milli: 1_250,
            attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
                .unwrap(),
            predecessor: None,
            host_lineage: worth_ui_host_contract::UiHostPresentationLineageIdentity::from_certification_host_session(1).unwrap(),
        },
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn basis_for_lineage(
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    host_lineage: worth_ui_host_contract::UiHostPresentationLineageIdentity,
    mounted_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    complete: bool,
    mechanics: Box<[WorthUiPresentationMechanicBasisInput]>,
    binding_pins: Box<[WorthUiPresentationPinBasis]>,
    pin_additions: Box<[WorthUiPresentationPinBasis]>,
    pin_releases: Box<[WorthUiPresentationPinBasis]>,
) -> WorthUiPresentationRequestBasis {
    WorthUiPresentationRequestBasis::from_runtime_correspondence(
        WorthUiPresentationRequestBasisInput {
            mounted_frame,
            semantic_surface,
            host_surface,
            binding,
            complete,
            mechanics,
            removed_mechanics: Box::new([]),
            binding_pins,
            pin_additions,
            pin_releases,
            dpi_milli: 1_250,
            attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
                .unwrap(),
            predecessor,
            host_lineage,
        },
    )
    .unwrap()
}

pub(super) fn mechanic(
    slot: u16,
    layout: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
) -> WorthUiPresentationMechanicBasisInput {
    let mounted_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let paint = worth_ui_host_contract::UiMountedTextForegroundSpan::from_runtime_mounting(
        worth_ui_host_contract::UiTextOriginalRange::new(0, 4).unwrap(),
        worth_ui_host_contract::UiMountedRgba8::new(1, 2, 3, 255),
        worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]),
    );
    WorthUiPresentationMechanicBasisInput {
        mounted_instance,
        mechanic:
            worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
                mounted_instance,
                slot,
                None,
            ),
        content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
            .unwrap(),
        content: std::sync::Arc::from("text"),
        layout,
        layout_request:
            worth_ui_host_contract::UiQualifiedTextLayoutRequestIdentity::from_text_mechanics(
                [2; 32],
            ),
        layout_width: worth_ui_host_contract::UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
        paint_spans: vec![WorthUiPresentationPaintSpanBasis::from_mounted(paint)]
            .into_boxed_slice(),
        raster_keys: vec![raster_key()].into_boxed_slice(),
        text_scale: worth_ui_host_contract::UiTextScaleGeneration::new(2).unwrap(),
    }
}

pub(super) fn raster_key() -> worth_ui_host_contract::UiGlyphRasterKey {
    worth_ui_host_contract::UiGlyphRasterKey::from_text_mechanics(
        worth_ui_host_contract::UiGlyphRasterKeyInput {
            font_collection: worth_ui_host_contract::UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage:
                worth_ui_host_contract::UiFontCollectionLineageIdentity::from_text_mechanics(
                    [4; 32],
                ),
            profile: worth_ui_host_contract::UiTextProfileGeneration::new(1).unwrap(),
            face: worth_ui_host_contract::UiQualifiedFontFaceIdentity::from_text_mechanics(
                [1; 32], 0,
            ),
            glyph_id: 1,
            variations: worth_ui_host_contract::UiGlyphVariationCoordinates::empty(),
            palette: worth_ui_host_contract::UiGlyphRasterPalette::new(0),
            size: worth_ui_host_contract::UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
            source: worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline,
            dpi_milli: 1_250,
            origin: worth_ui_host_contract::UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        },
    )
    .unwrap()
}
