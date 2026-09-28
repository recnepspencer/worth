use std::sync::atomic::{AtomicU64, Ordering};

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterLane, UiGlyphRasterPinTransitionView,
};

use crate::native::text_atlas::{canonical_raster_key_bytes, UiNativeTextAtlasQualifiedCapacity};

static NEXT_RUNTIME_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiNativePhysicalSignalRuntimeIdentity(u64);

impl UiNativePhysicalSignalRuntimeIdentity {
    pub(crate) fn mint() -> Self {
        let value = NEXT_RUNTIME_IDENTITY.fetch_add(1, Ordering::Relaxed);
        assert!(value != 0, "physical Signal runtime identity exhausted");
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalAtlasRequestIdentity {
    runtime: UiNativePhysicalSignalRuntimeIdentity,
    sequence: u64,
    presentation_basis: UiNativePhysicalPresentationBasis,
    basis_digest: [u8; 32],
}

pub(crate) struct UiNativePhysicalAtlasRequestInput<'work> {
    pub(crate) runtime: UiNativePhysicalSignalRuntimeIdentity,
    pub(crate) sequence: u64,
    pub(crate) presentation_basis: UiNativePhysicalPresentationBasis,
    pub(crate) demands: &'work [UiGlyphRasterDemandBatchView<'work>],
    pub(crate) pins: UiGlyphRasterPinTransitionView<'work>,
}

impl UiNativePhysicalAtlasRequestIdentity {
    pub(crate) fn from_inputs(input: UiNativePhysicalAtlasRequestInput<'_>) -> Self {
        let basis_digest = atlas_basis_digest(input.presentation_basis, input.demands, input.pins);
        Self {
            runtime: input.runtime,
            sequence: input.sequence,
            presentation_basis: input.presentation_basis,
            basis_digest,
        }
    }

    pub(crate) const fn presentation_basis(self) -> UiNativePhysicalPresentationBasis {
        self.presentation_basis
    }

    pub(crate) const fn sequence(self) -> u64 {
        self.sequence
    }

    pub(crate) const fn basis_digest(self) -> [u8; 32] {
        self.basis_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalAtlasUploadIdentity {
    request: UiNativePhysicalAtlasRequestIdentity,
    pending: worth_ui_host_contract::UiGlyphRasterTransactionPending,
}

impl UiNativePhysicalAtlasUploadIdentity {
    pub(crate) const fn new(
        request: UiNativePhysicalAtlasRequestIdentity,
        pending: worth_ui_host_contract::UiGlyphRasterTransactionPending,
    ) -> Self {
        Self { request, pending }
    }

    pub(crate) const fn request(self) -> UiNativePhysicalAtlasRequestIdentity {
        self.request
    }

    pub(crate) const fn pending(self) -> worth_ui_host_contract::UiGlyphRasterTransactionPending {
        self.pending
    }
}

fn atlas_basis_digest(
    presentation_basis: UiNativePhysicalPresentationBasis,
    demands: &[UiGlyphRasterDemandBatchView<'_>],
    pins: UiGlyphRasterPinTransitionView<'_>,
) -> [u8; 32] {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui/native-physical-atlas-request/v2\0");
    digest_presentation_basis(&mut digest, presentation_basis);
    digest_demands(&mut digest, demands);
    digest_pins(&mut digest, b"add\0", pins.additions());
    digest_pins(&mut digest, b"release\0", pins.releases());
    digest_capacity(&mut digest, UiNativeTextAtlasQualifiedCapacity::QUALIFIED);
    digest.finalize().into()
}

fn digest_presentation_basis(
    digest: &mut UiCountedSha256,
    basis: UiNativePhysicalPresentationBasis,
) {
    let baseline = basis.baseline();
    digest.update(basis.host_session_identity().to_le_bytes());
    digest.update(basis.attempt().diagnostic_value().to_le_bytes());
    digest.update(basis.surface().diagnostic_value().to_le_bytes());
    digest.update(basis.binding().diagnostic_value().to_le_bytes());
    digest.update(
        baseline
            .semantic_surface_identity()
            .diagnostic_value()
            .to_le_bytes(),
    );
    digest.update(
        baseline
            .host_surface_identity()
            .diagnostic_value()
            .to_le_bytes(),
    );
    digest.update(
        baseline
            .binding_generation()
            .diagnostic_value()
            .to_le_bytes(),
    );
    digest.update(baseline.capability_generation().as_u64().to_le_bytes());
    digest.update(baseline.capability_profile_digest().to_le_bytes());
    digest.update([match baseline.presentation_mode() {
        worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay => 0,
        worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly => 1,
    }]);
    digest.update(baseline.transparent_rgba8());
    let cost = basis.production_cost();
    for value in [
        cost.source_instances(),
        cost.commands_considered(),
        cost.command_index_lookups(),
        cost.order_lookups(),
        cost.retained_command_scans(),
        cost.retained_command_clones(),
        cost.projection_rows_materialized(),
    ] {
        digest.update(value.to_le_bytes());
    }
}

fn digest_demands(digest: &mut UiCountedSha256, demands: &[UiGlyphRasterDemandBatchView<'_>]) {
    digest.update((demands.len() as u64).to_le_bytes());
    for demand in demands {
        digest.update(demand.identity().digest());
        digest.update(demand.layout_identity().digest());
        digest.update(demand.dpi_milli().to_le_bytes());
        digest.update(demand.text_scale_generation().get().to_le_bytes());
        digest.update([match demand.lane() {
            UiGlyphRasterLane::Ordinary => 0,
            UiGlyphRasterLane::Reconstruction => 1,
        }]);
        digest.update((demand.records().len() as u64).to_le_bytes());
        for record in demand.records() {
            digest.update(canonical_raster_key_bytes(record.key()));
            digest.update(record.attribution().layout().digest());
            digest.update(record.attribution().original_range().start().to_le_bytes());
            digest.update(record.attribution().original_range().end().to_le_bytes());
            digest.update(record.extent().width().to_le_bytes());
            digest.update(record.extent().height().to_le_bytes());
            digest.update(record.staged_bytes().to_le_bytes());
        }
    }
}

fn digest_pins(
    digest: &mut UiCountedSha256,
    domain: &[u8],
    pins: &[worth_ui_host_contract::UiGlyphRasterPinRequest],
) {
    digest.update(domain);
    digest.update((pins.len() as u64).to_le_bytes());
    for pin in pins {
        digest.update(pin.layout_identity().digest());
        digest.update(canonical_raster_key_bytes(pin.key()));
    }
}

fn digest_capacity(digest: &mut UiCountedSha256, capacity: UiNativeTextAtlasQualifiedCapacity) {
    for value in [
        capacity.alpha_pages(),
        capacity.alpha_width(),
        capacity.alpha_height(),
        capacity.color_pages(),
        capacity.color_width(),
        capacity.color_height(),
        capacity.entries(),
        capacity.maximum_glyph_width(),
        capacity.maximum_glyph_height(),
    ] {
        digest.update(value.to_le_bytes());
    }
    digest.update(capacity.texel_bytes().to_le_bytes());
    digest.update(capacity.staged_upload_bytes().to_le_bytes());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalPresentationIdentity {
    runtime: UiNativePhysicalSignalRuntimeIdentity,
    sequence: u64,
    basis: UiNativePhysicalPresentationBasis,
}

impl UiNativePhysicalPresentationIdentity {
    pub(crate) const fn new(
        runtime: UiNativePhysicalSignalRuntimeIdentity,
        sequence: u64,
        basis: UiNativePhysicalPresentationBasis,
    ) -> Self {
        Self {
            runtime,
            sequence,
            basis,
        }
    }

    pub(crate) const fn basis(self) -> UiNativePhysicalPresentationBasis {
        self.basis
    }

    pub(crate) const fn sequence(self) -> u64 {
        self.sequence
    }

    pub(in crate::native::physical_work_signal) const fn request_identity(
        self,
    ) -> UiNativePhysicalRequestIdentity {
        UiNativePhysicalRequestIdentity {
            sequence: self.sequence,
            presentation_basis: self.basis,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalPresentationBasis {
    host_session_identity: u64,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
    production_cost: worth_ui_host_contract::UiMountedPresentationProductionCost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native::physical_work_signal) struct UiNativePhysicalSignalSlotLineage {
    host_session_identity: u64,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
}

impl UiNativePhysicalPresentationBasis {
    pub(crate) fn from_view(
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> Self {
        Self {
            host_session_identity: view.host_session_identity(),
            attempt: view.attempt(),
            surface: view.requirement().semantic_surface(),
            host_surface: view.requirement().host_surface(),
            binding: view.requirement().binding(),
            baseline: view.requirement().baseline(),
            production_cost: view.presentation_work().production_cost(),
        }
    }

    pub(crate) fn from_pin_release(
        request: worth_ui_host_contract::UiMountedTextPinReleaseRequest,
    ) -> Self {
        let surface = request.surface();
        Self {
            host_session_identity: surface.host_session_identity(),
            attempt: request.attempt(),
            surface: surface.semantic_surface_identity(),
            host_surface: surface.host_surface_identity(),
            binding: surface.binding_generation(),
            baseline: surface.baseline_identity(),
            production_cost: Default::default(),
        }
    }

    pub(crate) const fn host_session_identity(self) -> u64 {
        self.host_session_identity
    }

    pub(crate) const fn attempt(
        self,
    ) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub(crate) const fn surface(self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) const fn host_surface(self) -> worth_ui_host_contract::UiHostSurfaceIdentity {
        self.host_surface
    }

    pub(crate) const fn binding(self) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        self.binding
    }

    pub(crate) const fn production_cost(
        self,
    ) -> worth_ui_host_contract::UiMountedPresentationProductionCost {
        self.production_cost
    }

    pub(in crate::native::physical_work_signal) const fn slot_lineage(
        self,
    ) -> UiNativePhysicalSignalSlotLineage {
        UiNativePhysicalSignalSlotLineage {
            host_session_identity: self.host_session_identity,
            surface: self.surface,
            host_surface: self.host_surface,
            binding: self.binding,
        }
    }

    const fn baseline(self) -> worth_ui_host_contract::UiHostSurfaceBaselineIdentity {
        self.baseline
    }

    #[cfg(test)]
    pub(crate) fn test() -> Self {
        Self::test_with_host_session(1)
    }

    #[cfg(test)]
    pub(crate) fn test_with_host_session(host_session_identity: u64) -> Self {
        let requirement = worth_ui_host_contract::UiMountedSurfaceBindingRequirement::new(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration::new(1),
            1,
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
        );
        Self {
            host_session_identity,
            attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
                .unwrap(),
            surface: requirement.semantic_surface(),
            host_surface: requirement.host_surface(),
            binding: requirement.binding(),
            baseline: requirement.baseline(),
            production_cost: Default::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_successor(self) -> Self {
        Self {
            attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
                .unwrap(),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native::physical_work_signal) struct UiNativePhysicalRequestIdentity {
    sequence: u64,
    presentation_basis: UiNativePhysicalPresentationBasis,
}

impl UiNativePhysicalRequestIdentity {
    pub(in crate::native::physical_work_signal) const fn new(
        sequence: u64,
        presentation_basis: UiNativePhysicalPresentationBasis,
    ) -> Self {
        Self {
            sequence,
            presentation_basis,
        }
    }

    pub(in crate::native::physical_work_signal) const fn sequence(self) -> u64 {
        self.sequence
    }

    pub(in crate::native::physical_work_signal) const fn presentation_basis(
        self,
    ) -> UiNativePhysicalPresentationBasis {
        self.presentation_basis
    }
}
