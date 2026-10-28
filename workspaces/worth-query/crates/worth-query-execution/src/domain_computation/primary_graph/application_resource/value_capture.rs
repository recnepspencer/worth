//! Authenticated scalar projection and the sole typed binding's admitted decode.
use super::{
    occurrence::RetainedResourceValue,
    provider::{WorthQueryResourceVerificationFailure, WorthQueryResourceVerifiedFrame},
};
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_stage, CaptureStop,
};
use worth_execution::{
    ChargedBytes, ExecutionRequest, LeaseDenial, MapKernelFailure, WorkCeilingDenial,
};
use worth_foundational::facade::{AspectValue, ValueCodecDenial};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial,
};

#[expect(
    dead_code,
    reason = "original typed refusal is retained opaquely; disclosure uses approved projections"
)]
pub(super) enum ResourceValueDenial {
    Verification(WorthQueryResourceVerificationFailure),
    Encoding(ApplicationValueEncodeDenial),
    Decoding(ApplicationValueDecodeDenial),
    Request(WorkCeilingDenial),
    ValueLimit,
    UnsupportedProfile,
    InvalidValue,
}

impl ChargedBytes for ResourceValueDenial {
    fn additional_charged_bytes(&self) -> u64 {
        let bytes = match self {
            Self::Encoding(cause) => cause.retained_heap_bytes(),
            Self::Decoding(cause) => cause.binding_identity().retained_heap_bytes(),
            Self::Verification(_)
            | Self::Request(_)
            | Self::ValueLimit
            | Self::UnsupportedProfile
            | Self::InvalidValue => 0,
        };
        u64::try_from(bytes).unwrap_or(u64::MAX)
    }
}

struct ProjectedScalar(AspectValue);
impl ChargedBytes for ProjectedScalar {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(self.0.owned_allocation_capacity_bytes()).unwrap_or(u64::MAX)
    }
}

impl ChargedBytes for WorthQueryResourceVerifiedFrame {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(self.value.owned_allocation_capacity_bytes()).unwrap_or(u64::MAX)
    }
}

pub(super) fn capture_input<Binding: ApplicationScalarValueBinding>(
    execution: &ExecutionRequest<'_, '_>,
    input: &Binding::Value,
    maximum_bytes: u64,
) -> Result<RetainedResourceValue, CaptureStop<ResourceValueDenial>> {
    let (ProjectedScalar(value), funding) = run_stage(execution, 0, |context, _| {
        checkpoint(context, 1)?;
        Binding::validate(input)
            .map_err(|_| MapKernelFailure::Domain(ResourceValueDenial::InvalidValue))?;
        let value = Binding::encode(input)
            .map_err(|cause| MapKernelFailure::Domain(ResourceValueDenial::Encoding(cause)))?;
        require_value_limit(&value, maximum_bytes).map_err(MapKernelFailure::Domain)?;
        Ok(ProjectedScalar(value))
    })?;
    Ok(RetainedResourceValue { value, funding })
}

pub(super) fn verify_frame(
    execution: &ExecutionRequest<'_, '_>,
    maximum_bytes: u64,
    verify: impl FnOnce() -> Result<
        WorthQueryResourceVerifiedFrame,
        WorthQueryResourceVerificationFailure,
    >,
) -> Result<
    (super::WorthQueryResourceFrameKind, RetainedResourceValue),
    CaptureStop<ResourceValueDenial>,
> {
    // Opaque authored output heaps are measured after the callback, before the
    // framework retains them. Byte ingress/scratch admission happened earlier.
    let (frame, funding) = run_stage(execution, 0, |context, _| {
        checkpoint(context, 1)?;
        let frame = verify()
            .map_err(|cause| MapKernelFailure::Domain(ResourceValueDenial::Verification(cause)))?;
        require_value_limit(&frame.value, maximum_bytes).map_err(MapKernelFailure::Domain)?;
        Ok(frame)
    })?;
    Ok((
        frame.kind,
        RetainedResourceValue {
            value: frame.value,
            funding,
        },
    ))
}

fn require_value_limit(value: &AspectValue, maximum_bytes: u64) -> Result<(), ResourceValueDenial> {
    // Both the actual scalar width and its retained backing must fit the
    // installed value envelope; a tiny authenticated transport cannot smuggle
    // an arbitrarily large projected scalar into a retained result slot.
    let width = value
        .semantic_byte_width()
        .max(value.owned_allocation_capacity_bytes());
    if u64::try_from(width).is_ok_and(|width| width <= maximum_bytes) {
        Ok(())
    } else {
        Err(ResourceValueDenial::ValueLimit)
    }
}

struct Checked;
impl ChargedBytes for Checked {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(super) fn validate_scalar<Binding: ApplicationReadableScalarValueBinding>(
    execution: &ExecutionRequest<'_, '_>,
    value: &AspectValue,
) -> Result<(), CaptureStop<ResourceValueDenial>> {
    with_scalar::<Binding, _>(execution, value, |_| ())
}

/// The actual decoder's admitted temporary backing lives through the borrowed
/// host callback. No arbitrary typed value is retained by the framework.
pub(super) fn with_scalar<Binding: ApplicationReadableScalarValueBinding, R>(
    execution: &ExecutionRequest<'_, '_>,
    value: &AspectValue,
    inspect: impl FnOnce(&Binding::Value) -> R,
) -> Result<R, CaptureStop<ResourceValueDenial>> {
    let mut output = None;
    let _ = run_stage(execution, 0, |context, mut memory| {
        let mut first = None;
        let mut admit = |work: usize, bytes: usize| {
            if first.is_some() {
                return Err(ValueCodecDenial::Admission);
            }
            let outcome = (|| {
                let work = u64::try_from(work)
                    .map_err(|_| WorkCeilingDenial::Admission(LeaseDenial::ChargedBytesOverflow))?;
                checkpoint(context, work).map_err(WorkCeilingDenial::Stopped)?;
                if let Some(memory) = memory.as_deref_mut() {
                    let bytes = u64::try_from(bytes)
                        .ok()
                        .and_then(|bytes| memory.bytes().checked_add(bytes))
                        .ok_or(WorkCeilingDenial::Admission(
                            LeaseDenial::ChargedBytesOverflow,
                        ))?;
                    memory.resize(bytes).map_err(|cause| {
                        WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(cause))
                    })?;
                }
                Ok(())
            })();
            outcome.map_err(|cause| {
                first = Some(cause);
                ValueCodecDenial::Admission
            })
        };
        if Binding::canonical_contract().is_none() {
            let profile = Binding::NATIVE_INPUT_ADMISSION.ok_or(MapKernelFailure::Domain(
                ResourceValueDenial::UnsupportedProfile,
            ))?;
            let bytes = u64::try_from(value.owned_allocation_capacity_bytes())
                .map_err(|_| MapKernelFailure::Domain(ResourceValueDenial::ValueLimit))?;
            let resources = profile
                .requirements(bytes)
                .ok_or(MapKernelFailure::Domain(ResourceValueDenial::ValueLimit))?;
            let work = usize::try_from(resources.work_units())
                .map_err(|_| MapKernelFailure::Domain(ResourceValueDenial::ValueLimit))?;
            let bytes = usize::try_from(resources.peak_memory_bytes())
                .map_err(|_| MapKernelFailure::Domain(ResourceValueDenial::ValueLimit))?;
            if admit(work, bytes).is_err() {
                return Err(MapKernelFailure::Domain(ResourceValueDenial::Request(
                    first.expect("refused native predecode retains its original request cause"),
                )));
            }
        }
        let decoded = Binding::decode_admitted(value, &mut admit);

        if let Some(cause) = first {
            return Err(MapKernelFailure::Domain(ResourceValueDenial::Request(
                cause,
            )));
        }
        let decoded = decoded
            .map_err(|cause| MapKernelFailure::Domain(ResourceValueDenial::Decoding(cause)))?;
        Binding::validate(&decoded)
            .map_err(|_| MapKernelFailure::Domain(ResourceValueDenial::InvalidValue))?;
        output = Some(inspect(&decoded));
        drop(decoded);
        Ok(Checked)
    })?;
    Ok(output.expect("successful admitted decode invoked its borrowed callback"))
}
