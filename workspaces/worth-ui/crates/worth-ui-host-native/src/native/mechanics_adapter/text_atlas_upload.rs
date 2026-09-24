//! Replaceable external upload port for the native text-atlas transaction.

use crate::native::text_atlas::{
    UiNativeGpuAtlasKind, UiNativeTextAtlasGpuBatchUpload, UiNativeTextAtlasGpuPages,
    UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasUpload,
};
use crate::native::UiNativeHostState;

#[cfg(test)]
use super::text_atlas_physical_ownership::physical_ownership_counts_match;
use super::text_atlas_physical_ownership::physical_ownership_matches;
use super::text_atlas_settlement::map_denial;
use worth_ui_host_contract::UiGlyphRasterSource;
use worth_ui_host_contract::UiGlyphRasterTransactionDenial;

#[derive(Debug)]
pub(super) enum GpuUploadFailure {
    BeforeEffects(UiGlyphRasterTransactionDenial),
    Indeterminate,
}

pub(super) struct CorrelatedGpuUploadObservation {
    pub(super) external: Result<
        crate::native::text_atlas::UiNativeTextAtlasExternalOutcome,
        UiGlyphRasterTransactionDenial,
    >,
    pub(super) signal:
        crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation,
}

pub(super) trait UiNativeTextAtlasUploadPort {
    fn upload(
        &mut self,
        state: &mut UiNativeHostState,
        plan: &UiNativeTextAtlasTransactionPlan,
        uploads: &[UiNativeTextAtlasUpload],
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    ) -> CorrelatedGpuUploadObservation;
}

pub(super) struct RealTextAtlasUploadPort;

impl UiNativeTextAtlasUploadPort for RealTextAtlasUploadPort {
    fn upload(
        &mut self,
        state: &mut UiNativeHostState,
        plan: &UiNativeTextAtlasTransactionPlan,
        uploads: &[UiNativeTextAtlasUpload],
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    ) -> CorrelatedGpuUploadObservation {
        let UiNativeHostState {
            presentation_owners,
            text_atlas_gpu,
            resources,
            ..
        } = state;
        let Some(crate::native::UiNativePresentationOwners { device, .. }) =
            presentation_owners.as_ref()
        else {
            return correlated_failure(
                basis,
                GpuUploadFailure::BeforeEffects(UiGlyphRasterTransactionDenial::Unsupported),
            );
        };
        submit_correlated_upload(CorrelatedUploadInput {
            gpu: text_atlas_gpu,
            resources,
            device: device.state().device(),
            queue: device.state().queue(),
            plan,
            uploads,
            basis,
        })
    }
}

pub(super) struct CorrelatedUploadInput<'input> {
    pub(super) gpu: &'input mut Option<UiNativeTextAtlasGpuPages>,
    pub(super) resources: &'input mut crate::native::UiNativeResourceRegistry,
    pub(super) device: &'input wgpu::Device,
    pub(super) queue: &'input wgpu::Queue,
    pub(super) plan: &'input UiNativeTextAtlasTransactionPlan,
    pub(super) uploads: &'input [UiNativeTextAtlasUpload],
    pub(super) basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
}

pub(super) fn submit_correlated_upload(
    input: CorrelatedUploadInput<'_>,
) -> CorrelatedGpuUploadObservation {
    let gpu = input.gpu.get_or_insert_with(UiNativeTextAtlasGpuPages::new);
    if let Err(denial) =
        gpu.bind_transaction_correlation(input.plan.transaction_identity(), input.basis)
    {
        return correlated_failure(
            input.basis,
            GpuUploadFailure::BeforeEffects(map_denial(denial)),
        );
    }
    let result = RealTextAtlasUploadPort.submit_context(
        &mut UiNativeTextAtlasUploadContext {
            gpu,
            device: input.device,
            queue: input.queue,
            resources: input.resources,
        },
        UploadRequest {
            plan: input.plan,
            uploads: input.uploads,
        },
    );
    match result {
        Ok(())
            if physical_ownership_matches(
                input.plan.candidate_page_counts(),
                input
                    .gpu
                    .as_ref()
                    .expect("submitted atlas GPU owner remains installed"),
                input.resources,
            ) =>
        {
            correlated_submission(input.gpu, input.plan, input.basis)
        }
        Ok(()) => correlated_failure(input.basis, GpuUploadFailure::Indeterminate),
        Err(failure) => {
            if matches!(failure, GpuUploadFailure::BeforeEffects(_)) {
                input
                    .gpu
                    .as_mut()
                    .expect("upload correlation owner remains installed")
                    .release_transaction_correlation(input.plan.transaction_identity());
            }
            correlated_failure(input.basis, failure)
        }
    }
}

fn correlated_submission(
    gpu: &mut Option<UiNativeTextAtlasGpuPages>,
    plan: &UiNativeTextAtlasTransactionPlan,
    basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
) -> CorrelatedGpuUploadObservation {
    let transaction = plan.transaction_identity();
    let owner = gpu
        .as_mut()
        .expect("upload correlation owner remains installed");
    let status = if owner.transaction_pending(transaction) {
        crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending
    } else {
        owner.release_transaction_correlation(transaction);
        crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed
    };
    CorrelatedGpuUploadObservation {
        external: Ok(crate::native::text_atlas::UiNativeTextAtlasExternalOutcome::Submitted),
        signal: basis.observe(status),
    }
}

pub(super) fn correlated_failure(
    basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    failure: GpuUploadFailure,
) -> CorrelatedGpuUploadObservation {
    match failure {
        GpuUploadFailure::BeforeEffects(denial) => CorrelatedGpuUploadObservation {
            external: Err(denial),
            signal: basis.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::RejectedAfterRasterization,
            ),
        },
        GpuUploadFailure::Indeterminate => CorrelatedGpuUploadObservation {
            external: Ok(crate::native::text_atlas::UiNativeTextAtlasExternalOutcome::EffectsIndeterminate),
            signal: basis.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::EffectsIndeterminate,
            ),
        },
    }
}

impl RealTextAtlasUploadPort {
    fn submit_context(
        &mut self,
        context: &mut UiNativeTextAtlasUploadContext<'_>,
        request: UploadRequest<'_>,
    ) -> Result<(), GpuUploadFailure> {
        context.submit(request)
    }
}

struct UiNativeTextAtlasUploadContext<'context> {
    gpu: &'context mut UiNativeTextAtlasGpuPages,
    device: &'context wgpu::Device,
    queue: &'context wgpu::Queue,
    resources: &'context mut crate::native::UiNativeResourceRegistry,
}

struct UploadRequest<'request> {
    plan: &'request UiNativeTextAtlasTransactionPlan,
    uploads: &'request [UiNativeTextAtlasUpload],
}

struct ValidatedUpload {
    kind: UiNativeGpuAtlasKind,
    page: u32,
    origin: [u32; 2],
}

impl<'context> UiNativeTextAtlasUploadContext<'context> {
    fn submit(&mut self, request: UploadRequest<'_>) -> Result<(), GpuUploadFailure> {
        submit_uploads(self, request)
    }
}

trait AtlasUploadOperations {
    fn page_count(&self, kind: UiNativeGpuAtlasKind) -> usize;
    fn ensure_page(
        &mut self,
        kind: UiNativeGpuAtlasKind,
    ) -> Result<(), crate::native::text_atlas::UiNativeTextAtlasDenial>;
    fn upload_batch(
        &mut self,
        transaction: u64,
        validated: &[ValidatedUpload],
        uploads: &[UiNativeTextAtlasUpload],
    ) -> Result<(), crate::native::text_atlas::UiNativeTextAtlasDenial>;
}

impl AtlasUploadOperations for UiNativeTextAtlasUploadContext<'_> {
    fn page_count(&self, kind: UiNativeGpuAtlasKind) -> usize {
        self.gpu.page_count(kind)
    }

    fn ensure_page(
        &mut self,
        kind: UiNativeGpuAtlasKind,
    ) -> Result<(), crate::native::text_atlas::UiNativeTextAtlasDenial> {
        self.gpu.ensure_page(self.device, self.resources, kind)
    }

    fn upload_batch(
        &mut self,
        transaction: u64,
        validated: &[ValidatedUpload],
        uploads: &[UiNativeTextAtlasUpload],
    ) -> Result<(), crate::native::text_atlas::UiNativeTextAtlasDenial> {
        let requests = validated
            .iter()
            .zip(uploads)
            .map(|(validated, upload)| UiNativeTextAtlasGpuBatchUpload {
                kind: validated.kind,
                page: validated.page,
                origin: validated.origin,
                upload,
            })
            .collect::<Vec<_>>();
        self.gpu
            .upload_batch_for_transaction(
                self.device,
                self.queue,
                self.resources,
                transaction,
                &requests,
            )
            .map(|_| ())
    }
}

fn submit_uploads(
    operations: &mut impl AtlasUploadOperations,
    request: UploadRequest<'_>,
) -> Result<(), GpuUploadFailure> {
    let validated = request
        .uploads
        .iter()
        .map(|upload| validate_upload(request.plan, upload))
        .collect::<Result<Vec<_>, _>>()?;
    let mut effects_started = false;
    for placement in &validated {
        while operations.page_count(placement.kind)
            <= usize::try_from(placement.page).unwrap_or(usize::MAX)
        {
            operations
                .ensure_page(placement.kind)
                .map_err(|denial| upload_failure(effects_started, denial))?;
            effects_started = true;
        }
    }
    operations
        .upload_batch(
            request.plan.transaction_identity(),
            &validated,
            request.uploads,
        )
        .map_err(|denial| upload_failure(effects_started, denial))
}

fn validate_upload(
    plan: &UiNativeTextAtlasTransactionPlan,
    upload: &UiNativeTextAtlasUpload,
) -> Result<ValidatedUpload, GpuUploadFailure> {
    let (page, origin) =
        plan.placement_for(upload.key())
            .ok_or(GpuUploadFailure::BeforeEffects(
                UiGlyphRasterTransactionDenial::RasterBatchMismatch,
            ))?;
    let kind = match upload.key().source() {
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => {
            UiNativeGpuAtlasKind::Color
        }
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => {
            UiNativeGpuAtlasKind::Alpha
        }
    };
    Ok(ValidatedUpload { kind, page, origin })
}

pub(super) fn upload_failure(
    submitted: bool,
    denial: crate::native::text_atlas::UiNativeTextAtlasDenial,
) -> GpuUploadFailure {
    if submitted {
        GpuUploadFailure::Indeterminate
    } else {
        GpuUploadFailure::BeforeEffects(map_denial(denial))
    }
}

#[cfg(test)]
#[path = "text_atlas_upload_classifier_tests.rs"]
mod classifier_tests;

#[cfg(test)]
#[path = "text_atlas_upload_tests.rs"]
pub(super) mod tests;
