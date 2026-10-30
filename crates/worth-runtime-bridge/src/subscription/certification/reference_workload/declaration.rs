use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionReferenceWorkloadRejection, BridgeSubscriptionReferenceWorkloadRejectionKind,
};
use crate::subscription::certification::{
    BridgeSubscriptionReferenceWorkloadLaneKind, BridgeSubscriptionReferenceWorkloadLaneRequest,
    BridgeSubscriptionReferenceWorkloadManifestSealed,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionReferenceWorkloadDeclaration {
    manifest_digest: Arc<str>,
    lane_requests: Vec<BridgeSubscriptionReferenceWorkloadLaneRequest>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionReferenceWorkloadDeclaration {
    pub(crate) fn plan(
        manifest: &BridgeSubscriptionReferenceWorkloadManifestSealed,
        lane_requests: Vec<BridgeSubscriptionReferenceWorkloadLaneRequest>,
    ) -> Result<Self, BridgeSubscriptionReferenceWorkloadRejection> {
        let mut lane_requests = lane_requests;
        lane_requests.sort_by(|left, right| {
            left.lane_kind()
                .cmp(&right.lane_kind())
                .then_with(|| left.family_kind().cmp(&right.family_kind()))
        });
        lane_requests.dedup();
        if lane_requests.len() < 2 {
            return Err(BridgeSubscriptionReferenceWorkloadRejection::new(
                BridgeSubscriptionReferenceWorkloadRejectionKind::InsufficientLaneSet,
                None,
                "at least two unique lanes are required for cross-lane certification",
            ));
        }
        validate_manifest_lanes(manifest, &lane_requests)?;
        if !lane_requests.iter().any(|lane| {
            lane.lane_kind() == BridgeSubscriptionReferenceWorkloadLaneKind::AuthoritativeLive
        }) {
            return Err(BridgeSubscriptionReferenceWorkloadRejection::new(
                BridgeSubscriptionReferenceWorkloadRejectionKind::MissingAuthoritativeControlLane,
                None,
                "reference workloads require the authoritative live lane as the comparison control",
            ));
        }
        let lane_basis = lane_requests
            .iter()
            .map(|request| {
                format!(
                    "{}:{}",
                    request.lane_kind().as_str(),
                    request.family_kind().as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-reference-workload-declaration|manifest={}|lanes={lane_basis}",
            manifest.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            manifest_digest: Arc::from(manifest.digest()),
            lane_requests,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-reference-workload-declaration:sha256:{digest:x}"
            )),
        })
    }

    pub fn manifest_digest(&self) -> &str {
        self.manifest_digest.as_ref()
    }

    pub fn lane_requests(&self) -> &[BridgeSubscriptionReferenceWorkloadLaneRequest] {
        &self.lane_requests
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

fn validate_manifest_lanes(
    manifest: &BridgeSubscriptionReferenceWorkloadManifestSealed,
    lane_requests: &[BridgeSubscriptionReferenceWorkloadLaneRequest],
) -> Result<(), BridgeSubscriptionReferenceWorkloadRejection> {
    for request in lane_requests {
        if !manifest
            .lane_ids()
            .iter()
            .any(|lane_id| lane_id.as_ref() == request.lane_kind().as_str())
        {
            return Err(BridgeSubscriptionReferenceWorkloadRejection::new(
                BridgeSubscriptionReferenceWorkloadRejectionKind::LaneNotDeclaredByManifest,
                Some(request.lane_kind()),
                "requested lane is absent from the sealed manifest",
            ));
        }
    }
    Ok(())
}
