use worth_ui_inspection::{
    UiVisualArtifactPolicy, UiVisualSnapshotDenial, UiVisualSnapshotOmission,
    UiVisualSnapshotRelation,
};

use crate::inspection::visual_snapshot::UiVisualTarget;

use super::UiVisualCaptureFailure;

struct UiHostPixelValidation {
    request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    relation: UiVisualSnapshotRelation,
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
    pixels: Option<worth_ui_host_contract::UiHostPixelArtifact>,
    redaction: worth_ui_inspection::UiVisualPixelRedaction,
}

struct UiCapturedResourceSettlement {
    pixels: Option<worth_ui_inspection::UiVisualPixelArtifact>,
    resource_lease: crate::inspection::visual_snapshot::UiVisualSnapshotResourceLease,
}

struct UiCapturedResourceMeasurement {
    usage: crate::inspection::visual_snapshot::UiVisualRetainedResourceUsage,
    cost: worth_ui_inspection::UiVisualInspectionCostReceipt,
}

pub(super) fn seal_host_capture<Target, Policy>(
    observed: crate::inspection::visual_snapshot::UiObservedHostVisualCapture<Target, Policy>,
    relation: UiVisualSnapshotRelation,
    inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
) -> Result<
    crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Policy::CapturedPosture>,
    UiVisualCaptureFailure,
>
where
    Target: UiVisualTarget,
    Policy: UiVisualArtifactPolicy,
{
    let mut observed = observed.into_parts();
    let request = observed.requested.host_request();
    let observation = observed.take_observation();
    let (affinity, transform, regions, host_pixels) = observation.into_parts();
    if affinity.request() != request.identity() || affinity.copy_epoch() != request.expected_epoch()
    {
        return Err(UiVisualCaptureFailure::Indeterminate(
            worth_ui_inspection::UiVisualSnapshotIndeterminate::CaptureAffinity,
        ));
    }
    if !coordinate_transform_is_valid(transform) {
        return Err(UiVisualCaptureFailure::Denied(
            UiVisualSnapshotDenial::InvalidCoordinateTransform,
        ));
    }
    let spatial = crate::inspection::visual_snapshot::validate_and_index(
        observed.requested.capture_identity(),
        observed.requested.visual_regions(),
        &regions,
        transform,
    )
    .map_err(map_spatial_denial)?;
    let (visible_index, hit_test_index, spatial_cost) = spatial.into_parts();
    let pixels = validate_host_pixels(
        Policy::PIXELS_REQUESTED,
        Policy::PIXELS_REQUIRED,
        UiHostPixelValidation {
            request,
            relation,
            transform,
            pixels: host_pixels,
            redaction: inspection_policy.disclosure().pixel_redaction(),
        },
    )?;
    let indexed = observed.index(
        crate::inspection::visual_snapshot::UiValidatedHostVisualCapture::from_runtime_validation(
            crate::inspection::visual_snapshot::UiValidatedHostVisualCaptureInput {
                transform,
                pixels,
                visible_index,
                hit_test_index,
                spatial_cost,
            },
        ),
    );
    Ok(compose_snapshot_receipt(
        indexed,
        relation,
        inspection_policy,
    ))
}

fn coordinate_transform_is_valid(
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
) -> bool {
    let physical = transform.client_physical_dimensions();
    let logical = transform.viewport_logical_dimensions();
    let scale = transform.scale();
    let translation = transform.translation();
    if physical.contains(&0)
        || logical
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        || scale
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        || translation.iter().any(|value| !value.is_finite())
    {
        return false;
    }
    logical
        .into_iter()
        .zip(scale)
        .zip(physical)
        .all(|((logical, scale), physical)| {
            let projected = logical * scale;
            projected.is_finite()
                && match transform.rounding() {
                    worth_ui_host_contract::UiHostCoordinateRounding::PixelCenterNearest => {
                        projected.round() == physical as f32
                    }
                    worth_ui_host_contract::UiHostCoordinateRounding::FloorEdges => {
                        projected.floor() == physical as f32
                    }
                }
        })
}

fn compose_snapshot_receipt<Target, Policy>(
    indexed: crate::inspection::visual_snapshot::UiIndexedVisualCapture<Target, Policy>,
    relation: UiVisualSnapshotRelation,
    inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
) -> crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Policy::CapturedPosture>
where
    Target: UiVisualTarget,
    Policy: UiVisualArtifactPolicy,
{
    let parts = indexed.into_parts();
    let transform = parts.transform;
    let identity = crate::inspection::visual_snapshot::UiVisualSnapshotIdentity::issued_by_runtime(
        parts.capture_identity,
    );
    let measurement = measure_captured_resources::<Policy>(&parts);
    let affinity = snapshot_affinity(identity, parts.presentation, relation);
    let resource = settle_captured_resource(
        parts.registration,
        parts.pixels,
        identity,
        measurement.usage,
    );
    let _host_request = parts.host_request;
    let [width, height] = transform.client_physical_dimensions();
    let captured_client_extent =
        worth_ui_inspection::UiClientPhysicalRect::new(0, 0, width, height)
            .expect("validated nonempty host dimensions form a full client extent");
    crate::inspection::visual_snapshot::UiVisualSnapshotReceipt::seal(
        crate::inspection::visual_snapshot::UiVisualSnapshotSealInput {
            session: parts.session,
            identity,
            parent_snapshot: None,
            captured_client_extent,
            presentation: parts.presentation,
            affinity,
            coordinates: super::super::coordinate_projection::from_host(transform),
            host_coordinate_transform: transform,
            pixel_artifact: resource.pixels,
            disclosure: parts.disclosure,
            cost: measurement.cost,
            query_budget: worth_ui_inspection::UiVisualQueryBudget::from_runtime_projection(
                inspection_policy.maximum_query_results(),
                inspection_policy.maximum_query_candidates(),
            ),
            visible_index: parts.visible_index,
            hit_test_index: parts.hit_test_index,
            identity_trace_basis: parts.identity_trace_basis,
            snapshot_lease: parts.snapshot_lease,
            resource_lease: resource.resource_lease,
        },
    )
}

fn measure_captured_resources<Policy>(
    parts: &crate::inspection::visual_snapshot::UiIndexedVisualCaptureParts<Policy>,
) -> UiCapturedResourceMeasurement
where
    Policy: UiVisualArtifactPolicy,
{
    let pixel_bytes = parts
        .pixels
        .as_ref()
        .map_or(0, |artifact| artifact.bytes().len() as u64);
    let structural_bytes =
        crate::inspection::visual_snapshot::structural_reservation::retained_snapshot_structure(
            &parts.snapshot_lease,
            &parts.identity_trace_basis,
            &parts.visible_index,
            &parts.hit_test_index,
        )
        .expect("admitted host structure bounds the validated representation");
    UiCapturedResourceMeasurement {
        usage: crate::inspection::visual_snapshot::UiVisualRetainedResourceUsage::new(
            pixel_bytes,
            structural_bytes,
        ),
        cost: capture_cost(
            parts.spatial_cost.region_records_examined() as u64,
            Policy::PIXELS_REQUESTED,
            parts.transform,
            pixel_bytes,
            structural_bytes,
        ),
    }
}

fn settle_captured_resource(
    registration: crate::inspection::visual_snapshot::UiVisualCaptureRegistrationLease,
    pixels: Option<worth_ui_inspection::UiVisualPixelArtifact>,
    identity: crate::inspection::visual_snapshot::UiVisualSnapshotIdentity,
    usage: crate::inspection::visual_snapshot::UiVisualRetainedResourceUsage,
) -> UiCapturedResourceSettlement {
    let resource_lease = registration.complete(identity.diagnostic_value(), usage);
    let pixels =
        pixels.map(|artifact| artifact.bind_runtime_validity(resource_lease.pixel_validity()));
    UiCapturedResourceSettlement {
        pixels,
        resource_lease,
    }
}

pub(super) fn snapshot_affinity(
    identity: crate::inspection::visual_snapshot::UiVisualSnapshotIdentity,
    basis: crate::inspection::visual_snapshot::UiVisualSurfaceCaptureBasis,
    relation: UiVisualSnapshotRelation,
) -> worth_ui_inspection::UiVisualSnapshotAffinity {
    worth_ui_inspection::UiVisualSnapshotAffinity::from_runtime_projection(
        [
            identity.diagnostic_value(),
            basis.presentation_attempt.diagnostic_value(),
            basis.frame.diagnostic_value(),
            basis.semantic_surface.diagnostic_value(),
            basis.host_surface.diagnostic_value(),
            basis.binding.diagnostic_value(),
            basis.epoch.diagnostic_value(),
        ],
        relation,
    )
}

#[inline(never)]
fn capture_cost(
    region_records_examined: u64,
    pixels_requested: bool,
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
    pixel_bytes: u64,
    retained_structural_bytes: u64,
) -> worth_ui_inspection::UiVisualInspectionCostReceipt {
    let requested_bytes = if pixels_requested {
        expected_rgba_bytes(transform).unwrap_or(0)
    } else {
        0
    };
    worth_ui_inspection::UiVisualInspectionCostReceipt::from_runtime_projection([
        region_records_examined,
        0,
        0,
        0,
        requested_bytes,
        pixel_bytes,
        pixel_bytes,
        1,
        0,
        1,
        retained_structural_bytes,
    ])
}

fn map_spatial_denial(
    denial: crate::inspection::visual_snapshot::UiSpatialValidationDenial,
) -> UiVisualCaptureFailure {
    match denial {
        crate::inspection::visual_snapshot::UiSpatialValidationDenial::ProtocolMismatch => {
            UiVisualCaptureFailure::Denied(UiVisualSnapshotDenial::ProtocolIncompatible)
        }
        crate::inspection::visual_snapshot::UiSpatialValidationDenial::InvalidGeometry => {
            UiVisualCaptureFailure::Denied(UiVisualSnapshotDenial::InvalidGeometry)
        }
    }
}

#[inline(never)]
fn validate_host_pixels(
    pixels_requested: bool,
    pixels_required: bool,
    input: UiHostPixelValidation,
) -> Result<Option<worth_ui_inspection::UiVisualPixelArtifact>, UiVisualCaptureFailure> {
    let Some(pixels) = input.pixels else {
        return missing_pixels(pixels_required, input.relation);
    };
    if !pixels_requested {
        return Err(UiVisualCaptureFailure::Denied(
            UiVisualSnapshotDenial::ProtocolIncompatible,
        ));
    }
    validate_pixel_shape(input.request, input.transform, pixels, input.redaction)
}

#[inline(never)]
fn missing_pixels(
    pixels_required: bool,
    relation: UiVisualSnapshotRelation,
) -> Result<Option<worth_ui_inspection::UiVisualPixelArtifact>, UiVisualCaptureFailure> {
    if !pixels_required {
        return Ok(None);
    }
    match relation {
        UiVisualSnapshotRelation::Current => Err(UiVisualCaptureFailure::Denied(
            UiVisualSnapshotDenial::ProtocolIncompatible,
        )),
        UiVisualSnapshotRelation::RetainedPredecessor | UiVisualSnapshotRelation::Historical => {
            Err(UiVisualCaptureFailure::Omitted(
                UiVisualSnapshotOmission::HistoricalPixelsUnavailable,
            ))
        }
    }
}

fn validate_pixel_shape(
    request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
    pixels: worth_ui_host_contract::UiHostPixelArtifact,
    redaction: worth_ui_inspection::UiVisualPixelRedaction,
) -> Result<Option<worth_ui_inspection::UiVisualPixelArtifact>, UiVisualCaptureFailure> {
    let (dimensions, stride, bytes, color_space) = pixels.into_parts();
    if dimensions != transform.client_physical_dimensions()
        || expected_rgba_bytes(transform) != Some(bytes.len() as u64)
        || u64::from(stride) != u64::from(dimensions[0]).saturating_mul(4)
        || bytes.len() as u64 > request.maximum_pixel_bytes()
    {
        return Err(UiVisualCaptureFailure::Denied(
            UiVisualSnapshotDenial::ProtocolIncompatible,
        ));
    }
    let color_space = match color_space {
        worth_ui_host_contract::UiHostPixelColorSpace::Srgb => {
            worth_ui_inspection::UiVisualPixelColorSpace::Srgb
        }
        worth_ui_host_contract::UiHostPixelColorSpace::AdapterDeclared => {
            worth_ui_inspection::UiVisualPixelColorSpace::AdapterDeclared
        }
    };
    Ok(Some(
        worth_ui_inspection::UiVisualPixelArtifact::from_runtime_projection(
            worth_ui_inspection::UiVisualNativePixelArtifactInput {
                dimensions,
                stride,
                bytes,
                color_space,
                redaction,
            },
        ),
    ))
}

fn expected_rgba_bytes(
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
) -> Option<u64> {
    let [width, height] = transform.client_physical_dimensions();
    u64::from(width)
        .checked_mul(u64::from(height))?
        .checked_mul(4)
}
