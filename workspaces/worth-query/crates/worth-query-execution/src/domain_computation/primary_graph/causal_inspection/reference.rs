use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::anchor::CausalObservationAnchor;
use super::inventory::{CausalEvidenceFamily, CausalEvidenceOwner};
use super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalEvidenceReferenceReceiptIdentity,
    CausalEvidenceReferenceResolutionCountersIdentity,
    CausalEvidenceReferenceResolutionDenialIdentity, CausalObservationAnchorDigest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReference {
    owner: CausalEvidenceOwner,
    family: CausalEvidenceFamily,
    reference_digest: CausalEvidenceReferenceDigest,
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl CausalEvidenceReference {
    pub(super) fn new(
        owner: CausalEvidenceOwner,
        family: CausalEvidenceFamily,
        reference_digest: CausalEvidenceReferenceDigest,
    ) -> Self {
        let evidence_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
                .field_shape(WorthQueryEvidenceTag::new("owner"), owner.as_str())
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("reference"),
                    reference_digest.evidence_identity(),
                )
                .seal();
        Self {
            owner,
            family,
            reference_digest,
            evidence_identity,
        }
    }

    pub fn owner(&self) -> CausalEvidenceOwner {
        self.owner
    }

    pub fn family(&self) -> CausalEvidenceFamily {
        self.family
    }

    pub fn reference_digest(&self) -> &CausalEvidenceReferenceDigest {
        &self.reference_digest
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceReceipt {
    receipt_identity: CausalEvidenceReferenceReceiptIdentity,
    anchor_digest: CausalObservationAnchorDigest,
    reference_set_digest: CausalEvidenceReferenceDigest,
    resolved_reference_count: usize,
    missing_reference_family_count: usize,
}

impl CausalEvidenceReferenceReceipt {
    pub(super) fn new(
        anchor_digest: CausalObservationAnchorDigest,
        reference_set_digest: CausalEvidenceReferenceDigest,
        resolved_reference_count: usize,
        missing_reference_family_count: usize,
    ) -> Self {
        let receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            anchor_digest.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("reference_set"),
            reference_set_digest.evidence_identity(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("resolved"),
            resolved_reference_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("missing"),
            missing_reference_family_count,
        )
        .seal()
        .into();
        Self {
            receipt_identity,
            anchor_digest,
            reference_set_digest,
            resolved_reference_count,
            missing_reference_family_count,
        }
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn anchor_digest(&self) -> &CausalObservationAnchorDigest {
        &self.anchor_digest
    }

    pub fn reference_set_digest(&self) -> &CausalEvidenceReferenceDigest {
        &self.reference_set_digest
    }

    pub fn resolved_reference_count(&self) -> usize {
        self.resolved_reference_count
    }

    pub fn missing_reference_family_count(&self) -> usize {
        self.missing_reference_family_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceSet {
    anchor: CausalObservationAnchor,
    references: Vec<CausalEvidenceReference>,
    reference_set_digest: CausalEvidenceReferenceDigest,
    receipt: CausalEvidenceReferenceReceipt,
}

impl CausalEvidenceReferenceSet {
    pub(super) fn new(
        anchor: CausalObservationAnchor,
        references: Vec<CausalEvidenceReference>,
        reference_set_digest: CausalEvidenceReferenceDigest,
        receipt: CausalEvidenceReferenceReceipt,
    ) -> Self {
        Self {
            anchor,
            references,
            reference_set_digest,
            receipt,
        }
    }

    pub fn anchor(&self) -> &CausalObservationAnchor {
        &self.anchor
    }

    pub fn references(&self) -> &[CausalEvidenceReference] {
        &self.references
    }

    pub fn reference_set_digest(&self) -> &CausalEvidenceReferenceDigest {
        &self.reference_set_digest
    }

    pub fn receipt(&self) -> &CausalEvidenceReferenceReceipt {
        &self.receipt
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceResolutionCounters {
    requested_family_count: usize,
    anchor_reference_width: usize,
    indexed_record_count: usize,
    index_lookup_count: usize,
    resolved_reference_count: usize,
    missing_required_reference_count: usize,
    bridge_record_unindexed_scan_count: usize,
    retained_record_scan_count: usize,
    runtime_graph_scan_count: usize,
    counter_identity: CausalEvidenceReferenceResolutionCountersIdentity,
}

impl CausalEvidenceReferenceResolutionCounters {
    pub(super) fn new(
        requested_family_count: usize,
        anchor_reference_width: usize,
        consulted_indexed_record_count: usize,
        index_lookup_count: usize,
        resolved_reference_count: usize,
        missing_required_reference_count: usize,
    ) -> Self {
        let counter_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceResolutionCounters,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("requested_family_count"),
            requested_family_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("anchor_reference_width"),
            anchor_reference_width,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("indexed_record_count"),
            consulted_indexed_record_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("index_lookup_count"),
            index_lookup_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("resolved_reference_count"),
            resolved_reference_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("missing_required_reference_count"),
            missing_required_reference_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_record_unindexed_scan_count"),
            0,
        )
        .field_usize(WorthQueryEvidenceTag::new("retained_record_scan_count"), 0)
        .field_usize(WorthQueryEvidenceTag::new("runtime_graph_scan_count"), 0)
        .seal()
        .into();
        Self {
            requested_family_count,
            anchor_reference_width,
            indexed_record_count: consulted_indexed_record_count,
            index_lookup_count,
            resolved_reference_count,
            missing_required_reference_count,
            bridge_record_unindexed_scan_count: 0,
            retained_record_scan_count: 0,
            runtime_graph_scan_count: 0,
            counter_identity,
        }
    }

    pub fn requested_family_count(&self) -> usize {
        self.requested_family_count
    }

    pub fn anchor_reference_width(&self) -> usize {
        self.anchor_reference_width
    }

    pub fn indexed_record_count(&self) -> usize {
        self.indexed_record_count
    }

    pub fn index_lookup_count(&self) -> usize {
        self.index_lookup_count
    }

    pub fn resolved_reference_count(&self) -> usize {
        self.resolved_reference_count
    }

    pub fn missing_required_reference_count(&self) -> usize {
        self.missing_required_reference_count
    }

    pub fn bridge_record_unindexed_scan_count(&self) -> usize {
        self.bridge_record_unindexed_scan_count
    }

    pub fn retained_record_scan_count(&self) -> usize {
        self.retained_record_scan_count
    }

    pub fn runtime_graph_scan_count(&self) -> usize {
        self.runtime_graph_scan_count
    }

    pub fn counter_snapshot(&self) -> &str {
        self.counter_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceResolutionDenial {
    anchor_digest: CausalObservationAnchorDigest,
    missing_families: Vec<CausalEvidenceFamily>,
    missing_indexed_reference_count: usize,
    failure_identity: CausalEvidenceReferenceResolutionDenialIdentity,
}

impl CausalEvidenceReferenceResolutionDenial {
    pub(super) fn new(
        anchor_digest: CausalObservationAnchorDigest,
        missing_families: Vec<CausalEvidenceFamily>,
        missing_indexed_reference_count: usize,
    ) -> Self {
        let failure_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceResolutionDenial,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            anchor_digest.evidence_identity(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("missing"),
            missing_families.iter().map(CausalEvidenceFamily::as_str),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("missing_indexed_reference_count"),
            missing_indexed_reference_count,
        )
        .seal()
        .into();
        Self {
            anchor_digest,
            missing_families,
            missing_indexed_reference_count,
            failure_identity,
        }
    }

    pub fn anchor_digest(&self) -> &CausalObservationAnchorDigest {
        &self.anchor_digest
    }

    pub fn missing_families(&self) -> &[CausalEvidenceFamily] {
        &self.missing_families
    }

    pub fn missing_indexed_reference_count(&self) -> usize {
        self.missing_indexed_reference_count
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalEvidenceReferenceResolution {
    Resolved {
        reference_set: CausalEvidenceReferenceSet,
        counters: CausalEvidenceReferenceResolutionCounters,
    },
    MissingRequiredEvidence {
        denial: CausalEvidenceReferenceResolutionDenial,
        counters: CausalEvidenceReferenceResolutionCounters,
    },
}

impl CausalEvidenceReferenceResolution {
    pub fn counters(&self) -> &CausalEvidenceReferenceResolutionCounters {
        match self {
            Self::Resolved { counters, .. } | Self::MissingRequiredEvidence { counters, .. } => {
                counters
            }
        }
    }

    pub fn resolved_reference_set(&self) -> Option<&CausalEvidenceReferenceSet> {
        match self {
            Self::Resolved { reference_set, .. } => Some(reference_set),
            Self::MissingRequiredEvidence { .. } => None,
        }
    }

    pub fn denial(&self) -> Option<&CausalEvidenceReferenceResolutionDenial> {
        match self {
            Self::MissingRequiredEvidence { denial, .. } => Some(denial),
            Self::Resolved { .. } => None,
        }
    }
}
