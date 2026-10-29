use worth_ui_inspection::{
    UiVisualArtifactPolicy, UiVisualInspectionCostReceipt, UiVisualPixelArtifact,
    UiVisualSnapshotOmission,
};

use super::UiVisualCaptureFailure;

struct UiRgbaCrop {
    left: u32,
    top: u32,
    height: u32,
    stride: u32,
}

struct UiDerivedSnapshotSeal {
    source: crate::inspection::visual_snapshot::UiRetainedVisualSnapshotSource,
    region: worth_ui_inspection::UiClientPhysicalRect,
    identity: crate::inspection::visual_snapshot::UiVisualSnapshotIdentity,
    pixels: Option<UiVisualPixelArtifact>,
    retained_structural_bytes: u64,
}

pub(super) fn seal_derived_region<Target, Policy>(
    pending: crate::inspection::visual_snapshot::UiPendingDerivedRegionCapture<Target, Policy>,
) -> Result<
    crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Policy::CapturedPosture>,
    UiVisualCaptureFailure,
>
where
    Policy: UiVisualArtifactPolicy,
{
    let source = pending.source;
    let pixels = derive_pixels(
        Policy::PIXELS_REQUESTED,
        Policy::PIXELS_REQUIRED,
        &source,
        pending.region,
    )?;
    let identity = crate::inspection::visual_snapshot::UiVisualSnapshotIdentity::issued_by_runtime(
        pending.capture_identity,
    );
    let retained_pixel_bytes = pixels
        .as_ref()
        .map_or(0, |artifact| artifact.bytes().len() as u64);
    let retained_structural_bytes = retained_structure(&source);
    let usage = crate::inspection::visual_snapshot::UiVisualRetainedResourceUsage::new(
        retained_pixel_bytes,
        retained_structural_bytes,
    );
    let (source, validity) = source.replace_registered_resource(identity, usage);
    let pixels = pixels.map(|artifact| artifact.bind_runtime_validity(validity));
    Ok(seal_derived_snapshot::<Policy>(UiDerivedSnapshotSeal {
        source,
        region: pending.region,
        identity,
        pixels,
        retained_structural_bytes,
    }))
}

fn seal_derived_snapshot<Policy>(
    input: UiDerivedSnapshotSeal,
) -> crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Policy::CapturedPosture>
where
    Policy: UiVisualArtifactPolicy,
{
    let source = input.source;
    let affinity = super::captured_artifact::snapshot_affinity(
        input.identity,
        source.presentation,
        source.evidence.affinity().relation(),
    );
    let parent_snapshot = source.identity;
    let retained_pixel_bytes = input
        .pixels
        .as_ref()
        .map_or(0, |artifact| artifact.bytes().len() as u64);
    crate::inspection::visual_snapshot::UiVisualSnapshotReceipt::seal(
        crate::inspection::visual_snapshot::UiVisualSnapshotSealInput {
            session: source.session,
            identity: input.identity,
            parent_snapshot: Some(parent_snapshot),
            captured_client_extent: input.region,
            presentation: source.presentation,
            affinity,
            coordinates: source.evidence.coordinates(),
            host_coordinate_transform: source.host_coordinate_transform,
            pixel_artifact: input.pixels,
            disclosure: source.evidence.disclosure(),
            cost: derived_cost(
                Policy::PIXELS_REQUESTED,
                input.region,
                retained_pixel_bytes,
                input.retained_structural_bytes,
            ),
            query_budget: source.evidence.query_budget(),
            visible_index: source
                .visible_index
                .rebind_snapshot(input.identity.diagnostic_value()),
            hit_test_index: source
                .hit_test_index
                .rebind_snapshot(input.identity.diagnostic_value()),
            identity_trace_basis: source.identity_trace_basis,
            snapshot_lease: source.snapshot_lease,
            resource_lease: source.resource_lease,
        },
    )
}

#[inline(never)]
fn derive_pixels(
    pixels_requested: bool,
    pixels_required: bool,
    source: &crate::inspection::visual_snapshot::UiRetainedVisualSnapshotSource,
    region: worth_ui_inspection::UiClientPhysicalRect,
) -> Result<Option<UiVisualPixelArtifact>, UiVisualCaptureFailure> {
    if !pixels_requested {
        return Ok(None);
    }
    let Some(parent) = source.pixel_artifact.as_ref() else {
        return if pixels_required {
            Err(UiVisualCaptureFailure::Omitted(
                UiVisualSnapshotOmission::HistoricalPixelsUnavailable,
            ))
        } else {
            Ok(None)
        };
    };
    let extent = source.captured_client_extent;
    let left = region.left() - extent.left();
    let top = region.top() - extent.top();
    let width = region.right() - region.left();
    let height = region.bottom() - region.top();
    let stride = width
        .checked_mul(4)
        .expect("admission bounds derived RGBA stride");
    let bytes = crop_rgba(
        parent,
        UiRgbaCrop {
            left,
            top,
            height,
            stride,
        },
    )?;
    Ok(Some(UiVisualPixelArtifact::from_runtime_derived_crop(
        worth_ui_inspection::UiVisualDerivedPixelArtifactInput {
            dimensions: [width, height],
            stride,
            bytes,
            color_space: parent.color_space(),
            redaction: parent.redaction(),
            parent_snapshot: source.identity.diagnostic_value(),
            client_origin: [region.left(), region.top()],
        },
    )))
}

fn crop_rgba(
    parent: &UiVisualPixelArtifact,
    crop: UiRgbaCrop,
) -> Result<Box<[u8]>, UiVisualCaptureFailure> {
    let output_len =
        usize::try_from(u64::from(crop.stride) * u64::from(crop.height)).map_err(|_| {
            UiVisualCaptureFailure::Denied(
                worth_ui_inspection::UiVisualSnapshotDenial::CapacityExceeded,
            )
        })?;
    let mut output = Vec::with_capacity(output_len);
    let row_bytes = usize::try_from(crop.stride).expect("admitted stride fits usize");
    for row in crop.top..crop.top + crop.height {
        let start =
            usize::try_from(u64::from(row) * u64::from(parent.stride()) + u64::from(crop.left) * 4)
                .expect("validated parent artifact offsets fit usize");
        let end = start + row_bytes;
        output.extend_from_slice(&parent.bytes()[start..end]);
    }
    Ok(output.into_boxed_slice())
}

fn retained_structure(
    source: &crate::inspection::visual_snapshot::UiRetainedVisualSnapshotSource,
) -> u64 {
    crate::inspection::visual_snapshot::structural_reservation::retained_snapshot_structure(
        &source.snapshot_lease,
        &source.identity_trace_basis,
        &source.visible_index,
        &source.hit_test_index,
    )
    .expect("a retained parent already proved its structural accounting")
}

#[inline(never)]
fn derived_cost(
    pixels_requested: bool,
    region: worth_ui_inspection::UiClientPhysicalRect,
    retained_pixel_bytes: u64,
    retained_structural_bytes: u64,
) -> UiVisualInspectionCostReceipt {
    let requested = if pixels_requested {
        u64::from(region.right() - region.left())
            .saturating_mul(u64::from(region.bottom() - region.top()))
            .saturating_mul(4)
    } else {
        0
    };
    UiVisualInspectionCostReceipt::from_runtime_projection([
        0,
        0,
        0,
        0,
        requested,
        0,
        retained_pixel_bytes,
        0,
        0,
        1,
        retained_structural_bytes,
    ])
}
