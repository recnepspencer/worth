use sha2::{Digest, Sha256};

use super::WorthQueryExecutionResourceRequest;

pub(super) fn canonical_resource_request_identity(
    request: &WorthQueryExecutionResourceRequest,
) -> String {
    let mut hasher = Sha256::new();
    let atomic = request.boundary() == super::WorthQueryExecutionBoundary::Atomic;
    let hash = |hasher: &mut Sha256, value: &str| {
        if atomic {
            hasher.update((value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        } else {
            hash(hasher, value);
        }
    };
    hash(
        &mut hasher,
        if atomic {
            "worth-query-resource-request-v2"
        } else {
            "worth-query-resource-request-v1"
        },
    );
    if atomic {
        hash(&mut hasher, request.boundary().as_str());
        hash(&mut hasher, &request.scale().iter().count().to_string());
    }
    for (axis, value) in request.scale().iter() {
        hash(&mut hasher, axis.as_str());
        hash(&mut hasher, &value.to_string());
    }
    if atomic {
        hash(&mut hasher, &request.limits().iter().count().to_string());
    }
    for (dimension, value) in request.limits().iter() {
        hash(&mut hasher, dimension.as_str());
        hash(&mut hasher, &value.to_string());
    }
    for mode in request.modes() {
        hash(&mut hasher, mode.as_str());
    }
    for degradation in request.degradations() {
        hash(&mut hasher, degradation.as_str());
    }
    for posture in request.partial_effect_postures() {
        hash(&mut hasher, posture.as_str());
    }
    for posture in request.yielded_state_postures() {
        hash(&mut hasher, posture.as_str());
    }
    for posture in request.retained_progress_postures() {
        hash(&mut hasher, posture.as_str());
    }
    hash(&mut hasher, request.cancellation_safe_point().as_str());
    format!("{:x}", hasher.finalize())
}

fn hash(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_le_bytes());
    hasher.update(value.as_bytes());
}
