use std::collections::BTreeMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    validate_aspect_value, AspectFieldLocator, AspectKey, AspectLocator, AspectMask, ProjectionMask,
};
use worth_proof::TransitionOutcome;

use crate::mapping::SubscriptionSliceKind;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::snapshot::{
    BridgeSnapshotReadError, SnapshotReadContract, SnapshotReadCorrelationId,
    SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadTarget,
    ValidatedSnapshotReadPacketResult, ValidatedSnapshotReadRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotReadRequest {
    correlation_id: SnapshotReadCorrelationId,
    correlation_discriminator: Option<Arc<str>>,
    entity_identity: Arc<str>,
    relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    target: SnapshotReadTarget,
    shape: SnapshotReadShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SnapshotReadShape {
    Coarse,
    SubscriptionSlice { slice_kind: SubscriptionSliceKind },
}

impl SnapshotReadRequest {
    pub fn for_coarse(
        entity_identity: impl Into<Arc<str>>,
        contract: SnapshotReadContract,
    ) -> Self {
        let entity_identity = entity_identity.into();
        let target = SnapshotReadTarget::whole_aspect(contract);
        let shape = SnapshotReadShape::Coarse;
        let canonical_basis =
            snapshot_read_request_canonical_basis(entity_identity.as_ref(), &target, &shape, None);
        Self {
            correlation_id: SnapshotReadCorrelationId::from_native_request_basis(
                canonical_basis.as_ref(),
            ),
            correlation_discriminator: None,
            entity_identity,
            relational_record_identity: None,
            target,
            shape,
        }
    }

    pub(crate) fn for_coarse_relational_record(
        entity_identity: impl Into<Arc<str>>,
        relational_record_identity: RelationalBridgeRecordIdentityParts,
        contract: SnapshotReadContract,
    ) -> Self {
        let mut request = Self::for_coarse(entity_identity, contract);
        request.relational_record_identity = Some(relational_record_identity);
        request
    }

    pub fn for_native_subscription_slice(
        entity_identity: impl Into<Arc<str>>,
        contract: SnapshotReadContract,
        aspect_locator: AspectLocator,
        field_locator: Option<AspectFieldLocator>,
        projection_mask: AspectMask<ProjectionMask>,
        slice_kind: SubscriptionSliceKind,
    ) -> Self {
        Self::from_native_subscription_slice(
            entity_identity,
            contract,
            aspect_locator,
            field_locator,
            projection_mask,
            slice_kind,
        )
    }

    pub(crate) fn from_native_subscription_slice(
        entity_identity: impl Into<Arc<str>>,
        contract: SnapshotReadContract,
        aspect_locator: AspectLocator,
        field_locator: Option<AspectFieldLocator>,
        projection_mask: AspectMask<ProjectionMask>,
        slice_kind: SubscriptionSliceKind,
    ) -> Self {
        let entity_identity = entity_identity.into();
        let target = SnapshotReadTarget::native_subscription_slice(
            contract,
            aspect_locator,
            field_locator,
            projection_mask,
        );
        let shape = SnapshotReadShape::SubscriptionSlice { slice_kind };
        let canonical_basis =
            snapshot_read_request_canonical_basis(entity_identity.as_ref(), &target, &shape, None);
        Self {
            correlation_id: SnapshotReadCorrelationId::from_native_request_basis(
                canonical_basis.as_ref(),
            ),
            correlation_discriminator: None,
            entity_identity,
            relational_record_identity: None,
            target,
            shape,
        }
    }

    pub(crate) fn from_native_subscription_slice_relational_record(
        entity_identity: impl Into<Arc<str>>,
        relational_record_identity: RelationalBridgeRecordIdentityParts,
        contract: SnapshotReadContract,
        aspect_locator: AspectLocator,
        field_locator: Option<AspectFieldLocator>,
        projection_mask: AspectMask<ProjectionMask>,
        slice_kind: SubscriptionSliceKind,
    ) -> Self {
        let mut request = Self::from_native_subscription_slice(
            entity_identity,
            contract,
            aspect_locator,
            field_locator,
            projection_mask,
            slice_kind,
        );
        request.relational_record_identity = Some(relational_record_identity);
        request
    }

    pub(crate) fn with_correlation_discriminator(
        mut self,
        correlation_discriminator: impl Into<Arc<str>>,
    ) -> Self {
        self.correlation_discriminator = Some(correlation_discriminator.into());
        self.correlation_id =
            SnapshotReadCorrelationId::from_native_request_basis(self.canonical_basis().as_ref());
        self
    }

    pub fn correlation_id(&self) -> &SnapshotReadCorrelationId {
        &self.correlation_id
    }

    pub fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
    }

    pub fn relational_record_identity_parts(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        self.relational_record_identity
    }

    pub fn aspect_key(&self) -> &AspectKey {
        self.target.aspect_key()
    }

    pub fn target(&self) -> &SnapshotReadTarget {
        &self.target
    }

    pub(crate) fn target_identity(&self) -> &crate::snapshot::SnapshotReadTargetIdentity {
        self.target.target_identity()
    }

    pub fn native_target_basis(&self) -> &str {
        self.target.native_target_basis()
    }

    pub fn slice_kind(&self) -> Option<&SubscriptionSliceKind> {
        match &self.shape {
            SnapshotReadShape::Coarse => None,
            SnapshotReadShape::SubscriptionSlice { slice_kind, .. } => Some(slice_kind),
        }
    }

    pub(crate) fn canonical_basis(&self) -> Arc<str> {
        snapshot_read_request_canonical_basis(
            self.entity_identity(),
            &self.target,
            &self.shape,
            self.correlation_discriminator.as_deref(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotReadPacket {
    reads: Vec<SnapshotReadRequest>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl SnapshotReadPacket {
    pub fn new(reads: Vec<SnapshotReadRequest>) -> Self {
        let read_basis = reads
            .iter()
            .map(SnapshotReadRequest::canonical_basis)
            .collect::<Vec<_>>()
            .join("|");
        let canonical_basis = Arc::<str>::from(format!(
            "snapshot-read-packet|count={}|reads={read_basis}",
            reads.len(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            reads,
            canonical_basis,
            digest: Arc::from(format!("snapshot-read-packet:sha256:{digest:x}")),
        }
    }

    pub fn reads(&self) -> &[SnapshotReadRequest] {
        &self.reads
    }

    pub(crate) fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

pub(crate) fn validate_snapshot_read_result_contract(
    packet: &SnapshotReadPacket,
    unvalidated_result: SnapshotReadPacketResult,
) -> Result<ValidatedSnapshotReadPacketResult, BridgeSnapshotReadError> {
    let (snapshot_identity, records) = unvalidated_result.into_parts();
    let mut record_lookup = BTreeMap::new();
    for record in records {
        let correlation_id = record.correlation_id().clone();
        if record_lookup
            .insert(correlation_id.clone(), record)
            .is_some()
        {
            return Err(BridgeSnapshotReadError::duplicate_record(correlation_id));
        }
    }

    if record_lookup.len() != packet.reads().len() {
        return Err(BridgeSnapshotReadError::record_count_mismatch(
            record_lookup.len(),
            packet.reads().len(),
        ));
    }

    let mut canonical_records = Vec::with_capacity(packet.reads().len());
    for read in packet.reads() {
        let Some(record) = record_lookup.remove(read.correlation_id()) else {
            return Err(BridgeSnapshotReadError::missing_record(
                read.correlation_id().clone(),
            ));
        };
        let validated_record = validate_snapshot_read_record(read, record)?;
        canonical_records.push(validated_record);
    }

    if let Some(extra_correlation_id) = record_lookup.into_keys().next() {
        return Err(BridgeSnapshotReadError::extra_record(extra_correlation_id));
    }

    Ok(ValidatedSnapshotReadPacketResult::validated(
        snapshot_identity,
        canonical_records,
    ))
}

fn validate_snapshot_read_record(
    read: &SnapshotReadRequest,
    record: SnapshotReadRecord,
) -> Result<ValidatedSnapshotReadRecord, BridgeSnapshotReadError> {
    read.target()
        .projection_contract()
        .admits_projection_mask(read.target().projection_mask())
        .map_err(|denial| {
            BridgeSnapshotReadError::projection_mask_rejected(
                read.correlation_id().clone(),
                read.target().contract().aspect_key().clone(),
                denial,
            )
        })?;

    let Some(read_value) = record.read_value_posture() else {
        return Ok(ValidatedSnapshotReadRecord::absent(
            record.correlation_id().clone(),
        ));
    };
    let validation = validate_aspect_value(
        read.target().contract().aspect_contract(),
        read_value.clone().into_validation_input(),
    );
    match validation {
        TransitionOutcome::Success(validated) => Ok(ValidatedSnapshotReadRecord::new(
            record.correlation_id().clone(),
            validated,
        )),
        TransitionOutcome::Denied(denial) => {
            Err(BridgeSnapshotReadError::aspect_contract_validation_denied(
                read.correlation_id().clone(),
                read.target().contract().aspect_key().clone(),
                denial,
            ))
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("aspect validation uses success or denied"),
    }
}

fn snapshot_read_request_canonical_basis(
    entity_identity: &str,
    target: &SnapshotReadTarget,
    shape: &SnapshotReadShape,
    correlation_discriminator: Option<&str>,
) -> Arc<str> {
    let basis = match shape {
        SnapshotReadShape::Coarse => format!(
            "snapshot-read-request|entity={entity_identity}|target={}|shape=coarse",
            target.target_identity().as_str(),
        ),
        SnapshotReadShape::SubscriptionSlice { slice_kind } => format!(
            "snapshot-read-request|entity={entity_identity}|target={}|shape=subscription-slice|slice-kind={}",
            target.target_identity().as_str(),
            canonical_subscription_slice_kind_label(slice_kind),
        ),
    };
    match correlation_discriminator {
        Some(discriminator) => {
            Arc::from(format!("{basis}|correlation-discriminator={discriminator}"))
        }
        None => Arc::from(basis),
    }
}

pub(crate) fn canonical_subscription_slice_kind_label(
    slice_kind: &SubscriptionSliceKind,
) -> &'static str {
    match slice_kind {
        SubscriptionSliceKind::SignalAspect => "signal-aspect",
        SubscriptionSliceKind::SignalField => "signal-field",
        SubscriptionSliceKind::SignalLens => "signal-lens",
        SubscriptionSliceKind::SignalRegion => "signal-region",
        SubscriptionSliceKind::SignalPartition => "signal-partition",
        SubscriptionSliceKind::SignalFacet => "signal-facet",
        SubscriptionSliceKind::SignalLifecycle => "signal-lifecycle",
        SubscriptionSliceKind::RegisteredCoarseWidening => "registered-coarse-widening",
    }
}

#[cfg(test)]
#[path = "packet_tests.rs"]
mod tests;
