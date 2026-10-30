use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::diagnostics::BridgeRouteRecord;
use crate::error::{BridgeContinuityError, BridgeContinuityErrorKind};
use crate::facade::BridgePlannedRoute;
use crate::routing::{BridgeRouteIdentity, BridgeSubscriptionSliceIdentity};

use super::BridgeContinuityAuthorityBasis;
mod prior_slice;

pub use prior_slice::PriorSubscriptionSlice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeContinuityCorrelationId {
    value: Arc<str>,
}

impl BridgeContinuityCorrelationId {
    fn from_planned_request_basis(canonical_basis: &str) -> Self {
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            value: Arc::from(format!("continuity-correlation:sha256:{digest:x}")),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlannedContinuityRequest {
    correlation_id: BridgeContinuityCorrelationId,
    prior_route_identity: BridgeRouteIdentity,
    prior_slice: PriorSubscriptionSlice,
}

impl BridgePlannedContinuityRequest {
    fn new(prior_route_identity: BridgeRouteIdentity, prior_slice: PriorSubscriptionSlice) -> Self {
        let canonical_basis = format!(
            "planned-continuity-request|route={}|{}",
            prior_route_identity.as_str(),
            prior_slice.canonical_basis()
        );
        Self {
            correlation_id: BridgeContinuityCorrelationId::from_planned_request_basis(
                &canonical_basis,
            ),
            prior_route_identity,
            prior_slice,
        }
    }

    pub fn correlation_id(&self) -> &BridgeContinuityCorrelationId {
        &self.correlation_id
    }

    pub fn prior_route_identity(&self) -> &BridgeRouteIdentity {
        &self.prior_route_identity
    }

    pub fn prior_slice(&self) -> &PriorSubscriptionSlice {
        &self.prior_slice
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePlannedContinuityRequestSet {
    prior_route_identity: BridgeRouteIdentity,
    authority_basis: BridgeContinuityAuthorityBasis,
    prior_subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    prior_slice_count: usize,
    requests: Arc<[BridgePlannedContinuityRequest]>,
    digest: Arc<str>,
}

impl BridgePlannedContinuityRequestSet {
    pub(crate) fn from_route_record(
        route_record: &BridgeRouteRecord,
    ) -> Result<Self, BridgeContinuityError> {
        let lineage_context = route_record
            .mapping_context()
            .lineage_context()
            .ok_or_else(|| {
                BridgeContinuityError::new(
                    BridgeContinuityErrorKind::MissingLineageContext,
                    "Bridge continuity planning requires an explicit lineage context in the mapping context.",
                )
            })?;
        let authority_basis = lineage_context.authority_basis().clone();
        if authority_basis.snapshot_identity() != route_record.source_snapshot() {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::LineageAuthorityMismatch,
                format!(
                    "Bridge continuity lineage context was bound to snapshot `{}` but route record source snapshot was `{}`.",
                    authority_basis.snapshot_identity().as_str(),
                    route_record.source_snapshot().as_str(),
                ),
            ));
        }
        if authority_basis.branch_identity() != route_record.source_branch() {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::LineageAuthorityMismatch,
                format!(
                    "Bridge continuity lineage context was bound to branch `{}` but route record source branch was `{}`.",
                    authority_basis.branch_identity().as_str(),
                    route_record.source_branch().as_str(),
                ),
            ));
        }
        let prior_route_identity = route_record.route_identity().clone();
        let prior_subscription_slice_identity = route_record.subscription_slice_identity().clone();
        let prior_slice_count = route_record.subscription_slices().len();
        let mut prior_slices = route_record
            .subscription_slices()
            .iter()
            .map(|slice| {
                PriorSubscriptionSlice::new(prior_subscription_slice_identity.clone(), slice)
            })
            .collect::<Vec<_>>();
        prior_slices.sort_by(|left, right| {
            left.logical_dedup_basis()
                .cmp(&right.logical_dedup_basis())
                .then_with(|| {
                    left.prior_subscription_slice_identity()
                        .as_str()
                        .cmp(right.prior_subscription_slice_identity().as_str())
                })
        });
        prior_slices
            .dedup_by(|left, right| left.logical_dedup_basis() == right.logical_dedup_basis());
        let requests = prior_slices
            .into_iter()
            .map(|prior_slice| {
                BridgePlannedContinuityRequest::new(prior_route_identity.clone(), prior_slice)
            })
            .collect::<Vec<_>>();
        let mut requests = requests;
        requests.sort_by(|left, right| {
            left.correlation_id()
                .as_str()
                .cmp(right.correlation_id().as_str())
        });
        let canonical_basis = format!(
            "planned-continuity-request-set|route={}|slice-set={}|authority={}|prior-slice-count={}|request-count={}|requests={}",
            prior_route_identity.as_str(),
            prior_subscription_slice_identity.as_str(),
            authority_basis.digest(),
            prior_slice_count,
            requests.len(),
            requests
                .iter()
                .map(|request| request.correlation_id().as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            prior_route_identity,
            authority_basis,
            prior_subscription_slice_identity,
            prior_slice_count,
            requests: Arc::from(requests),
            digest: Arc::from(format!("planned-continuity-set:sha256:{digest:x}")),
        })
    }

    pub(crate) fn from_planned_route(
        planned_route: &BridgePlannedRoute,
    ) -> Result<Self, BridgeContinuityError> {
        let lineage_context = planned_route
            .mapping_context()
            .lineage_context()
            .ok_or_else(|| {
                BridgeContinuityError::new(
                    BridgeContinuityErrorKind::MissingLineageContext,
                    "Bridge continuity planning requires an explicit lineage context in the mapping context.",
                )
            })?;
        let authority_basis = lineage_context.authority_basis().clone();
        if authority_basis.snapshot_identity() != planned_route.source_snapshot() {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::LineageAuthorityMismatch,
                "Bridge continuity lineage context snapshot did not match planned route source snapshot.",
            ));
        }
        if authority_basis.branch_identity() != planned_route.source_branch() {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::LineageAuthorityMismatch,
                "Bridge continuity lineage context branch did not match planned route source branch.",
            ));
        }
        let prior_route_identity = planned_route.route_identity().clone();
        let prior_subscription_slice_identity = planned_route
            .lowering_summary()
            .subscription_slice_identity()
            .clone();
        let prior_slice_count = planned_route.subscription_slices().len();
        let mut prior_slices = planned_route
            .subscription_slices()
            .slices()
            .iter()
            .map(|slice| {
                PriorSubscriptionSlice::new(prior_subscription_slice_identity.clone(), slice)
            })
            .collect::<Vec<_>>();
        prior_slices.sort_by(|left, right| {
            left.logical_dedup_basis()
                .cmp(&right.logical_dedup_basis())
                .then_with(|| {
                    left.prior_subscription_slice_identity()
                        .as_str()
                        .cmp(right.prior_subscription_slice_identity().as_str())
                })
        });
        prior_slices
            .dedup_by(|left, right| left.logical_dedup_basis() == right.logical_dedup_basis());
        let mut requests = prior_slices
            .into_iter()
            .map(|prior_slice| {
                BridgePlannedContinuityRequest::new(prior_route_identity.clone(), prior_slice)
            })
            .collect::<Vec<_>>();
        requests.sort_by(|left, right| {
            left.correlation_id()
                .as_str()
                .cmp(right.correlation_id().as_str())
        });
        let canonical_basis = format!(
            "planned-continuity-request-set|route={}|slice-set={}|authority={}|prior-slice-count={}|request-count={}|requests={}",
            prior_route_identity.as_str(),
            prior_subscription_slice_identity.as_str(),
            authority_basis.digest(),
            prior_slice_count,
            requests.len(),
            requests
                .iter()
                .map(|request| request.correlation_id().as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            prior_route_identity,
            authority_basis,
            prior_subscription_slice_identity,
            prior_slice_count,
            requests: Arc::from(requests),
            digest: Arc::from(format!("planned-continuity-set:sha256:{digest:x}")),
        })
    }

    pub fn prior_route_identity(&self) -> &BridgeRouteIdentity {
        &self.prior_route_identity
    }

    pub fn authority_basis(&self) -> &BridgeContinuityAuthorityBasis {
        &self.authority_basis
    }

    pub fn prior_subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.prior_subscription_slice_identity
    }

    pub fn prior_slice_count(&self) -> usize {
        self.prior_slice_count
    }

    pub fn requests(&self) -> &[BridgePlannedContinuityRequest] {
        &self.requests
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeEligibleContinuityRequestSet {
    planned: BridgePlannedContinuityRequestSet,
}

impl BridgeEligibleContinuityRequestSet {
    pub(crate) fn from_planned(
        planned: BridgePlannedContinuityRequestSet,
    ) -> Result<Self, BridgeContinuityError> {
        let mut correlation_ids = planned
            .requests()
            .iter()
            .map(|request| request.correlation_id().as_str())
            .collect::<Vec<_>>();
        let mut sorted = correlation_ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != correlation_ids.len() {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::InvalidContinuityRequestSet,
                "Bridge continuity request-set contained duplicate correlation ids after canonical planning.",
            ));
        }
        correlation_ids.sort_unstable();
        if correlation_ids != sorted {
            return Err(BridgeContinuityError::new(
                BridgeContinuityErrorKind::InvalidContinuityRequestSet,
                "Bridge continuity request-set was not emitted in canonical correlation order.",
            ));
        }
        Ok(Self { planned })
    }

    pub fn prior_route_identity(&self) -> &BridgeRouteIdentity {
        self.planned.prior_route_identity()
    }

    pub fn authority_basis(&self) -> &BridgeContinuityAuthorityBasis {
        self.planned.authority_basis()
    }

    pub fn prior_subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        self.planned.prior_subscription_slice_identity()
    }

    pub fn prior_slice_count(&self) -> usize {
        self.planned.prior_slice_count()
    }

    pub fn requests(&self) -> &[BridgePlannedContinuityRequest] {
        self.planned.requests()
    }

    pub fn digest(&self) -> &str {
        self.planned.digest()
    }
}
