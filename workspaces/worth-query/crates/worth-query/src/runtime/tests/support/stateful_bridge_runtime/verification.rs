use super::super::*;
use super::state::{NativeExternalRow, StatefulBridgeState};
use super::writes::native_external_field_path_for_touch;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};

pub(super) fn verify_existing_truth_assertion(
    state: &StatefulBridgeState,
    binding: &WorthQueryExistingTruthTargetBinding,
    aspects: &[WorthQueryAuthoredAspectMutation],
    snapshot_identity: WorthQuerySnapshotIdentity,
) -> Result<WorthQueryVerifiedExistingTruthAssertion, WorthQueryExistingTruthAssertionDenial> {
    let row = authoritative_row(state, binding).map_err(|message| {
        WorthQueryExistingTruthAssertionDenial::new(
            binding,
            WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
            None,
            None,
            None,
            message,
        )
    })?;
    for aspect in aspects {
        let aspect_touch = aspect.aspect_touch();
        if aspect.clears_existing_value() {
            return Err(WorthQueryExistingTruthAssertionDenial::new(
                binding,
                WorthQueryExistingTruthAssertionDenialKind::ClearAssertionUnsupported,
                Some(aspect_touch.clone()),
                None,
                None,
                "backend-verified assertions cannot clear authoritative truth",
            ));
        }
        let Some(found) = get_native_touch(row, &aspect_touch) else {
            return Err(WorthQueryExistingTruthAssertionDenial::new(
                binding,
                WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                Some(aspect_touch.clone()),
                Some(aspect.terminal_digest_material()),
                None,
                "authoritative truth did not contain the asserted aspect",
            ));
        };
        let Some(expected) = aspect.foundational_value() else {
            return Err(WorthQueryExistingTruthAssertionDenial::new(
                binding,
                WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                Some(aspect_touch.clone()),
                Some(aspect.terminal_digest_material()),
                None,
                "asserted aspect did not retain a native value",
            ));
        };
        if found != expected {
            return Err(WorthQueryExistingTruthAssertionDenial::new(
                binding,
                WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch,
                Some(aspect_touch.clone()),
                Some(aspect.terminal_digest_material()),
                Some(terminal_digest_from_aspect_value(found)),
                "authoritative truth did not match the asserted value",
            ));
        }
    }
    WorthQueryVerifiedExistingTruthAssertion::new(binding, aspects, snapshot_identity).map_err(
        |error| {
            WorthQueryExistingTruthAssertionDenial::new(
                binding,
                WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                None,
                None,
                None,
                error.to_string(),
            )
        },
    )
}

fn terminal_digest_from_aspect_value(value: &AspectValue) -> String {
    prepare_aspect_value_identity_basis(value)
        .as_str()
        .to_owned()
}

pub(super) fn probe_existing_truth(
    state: &StatefulBridgeState,
    request: &WorthQueryExistingTruthProbeRequest,
) -> Result<WorthQueryExistingTruthProbe, WorthQueryExistingTruthProbeDenial> {
    let row = authoritative_row(state, request.binding()).map_err(|message| {
        WorthQueryExistingTruthProbeDenial::new(
            request.binding(),
            WorthQueryExistingTruthProbeDenialKind::ResolvedTargetUnavailable,
            None,
            message,
        )
    })?;
    let mut fields = Vec::with_capacity(request.aspect_touches().len());
    for aspect_touch in request.aspect_touches() {
        let Some(value) = get_native_touch(row, aspect_touch) else {
            return Err(WorthQueryExistingTruthProbeDenial::new(
                request.binding(),
                WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
                Some(aspect_touch.clone()),
                "authoritative truth did not contain the probed aspect",
            ));
        };
        let field = WorthQueryExistingTruthProbeField::from_admitted_aspect_touch(
            aspect_touch.clone(),
            value.clone(),
        );
        fields.push(field.map_err(|cause| {
            WorthQueryExistingTruthProbeDenial::new(
                request.binding(),
                WorthQueryExistingTruthProbeDenialKind::EvidenceIdentityDerivationDenied(cause),
                None,
                "canonical digest refused probe field",
            )
        })?);
    }
    WorthQueryExistingTruthProbe::backend_verified(request, fields)
}

fn authoritative_row<'a>(
    state: &'a StatefulBridgeState,
    binding: &WorthQueryExistingTruthTargetBinding,
) -> Result<&'a NativeExternalRow, String> {
    let resolved_target_identity = binding
        .resolved_target_identity()
        .terminal_projection_for_reporting();
    let Some(collection) = state.collection_by_identity.get(&resolved_target_identity) else {
        return Err(format!(
            "resolved target `{}` is not present in authoritative truth",
            resolved_target_identity
        ));
    };
    state
        .rows_by_collection
        .get(collection)
        .and_then(|rows| rows.get(&resolved_target_identity))
        .ok_or_else(|| {
            format!(
                "resolved target `{}` is not present in authoritative truth",
                resolved_target_identity
            )
        })
}

fn get_native_touch<'a>(
    row: &'a NativeExternalRow,
    aspect_touch: &WorthQueryAspectTouch,
) -> Option<&'a AspectValue> {
    let field_path = native_external_field_path_for_touch(aspect_touch).ok()?;
    row.get(&field_path)
}
