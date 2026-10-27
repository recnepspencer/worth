use std::collections::BTreeMap;

use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::inventory::{CausalEvidenceFamily, CausalEvidenceOwner};
use super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalEvidenceReferenceIndexErrorIdentity,
    CausalEvidenceReferenceIndexIdentity, CausalEvidenceReferenceIndexRecordIdentity,
    CausalEvidenceReferenceInput,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceIndex {
    records: BTreeMap<
        CausalEvidenceFamily,
        BTreeMap<CausalEvidenceReferenceDigest, CausalEvidenceReferenceIndexRecord>,
    >,
    index_identity: CausalEvidenceReferenceIndexIdentity,
}

impl CausalEvidenceReferenceIndex {
    pub fn new(records: Vec<CausalEvidenceReferenceIndexRecord>) -> Self {
        let mut indexed_records = BTreeMap::<
            CausalEvidenceFamily,
            BTreeMap<CausalEvidenceReferenceDigest, CausalEvidenceReferenceIndexRecord>,
        >::new();
        for record in records {
            indexed_records
                .entry(record.family())
                .or_default()
                .insert(record.reference_digest().clone(), record);
        }
        let index_identity = index_identity(&indexed_records);
        Self {
            records: indexed_records,
            index_identity,
        }
    }

    pub(super) fn record_for_reference(
        &self,
        family: CausalEvidenceFamily,
        reference_digest: &CausalEvidenceReferenceDigest,
    ) -> Option<&CausalEvidenceReferenceIndexRecord> {
        self.records
            .get(&family)
            .and_then(|family_records| family_records.get(reference_digest))
    }

    pub fn record_count(&self) -> usize {
        self.records.values().map(BTreeMap::len).sum()
    }

    pub fn family_count(&self) -> usize {
        self.records.len()
    }

    pub fn index_digest(&self) -> &str {
        self.index_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceIndexRecord {
    owner: CausalEvidenceOwner,
    family: CausalEvidenceFamily,
    reference_digest: CausalEvidenceReferenceDigest,
    record_identity: CausalEvidenceReferenceIndexRecordIdentity,
}

impl CausalEvidenceReferenceIndexRecord {
    pub fn new(
        owner: CausalEvidenceOwner,
        family: CausalEvidenceFamily,
        reference_digest: CausalEvidenceReferenceDigest,
    ) -> Self {
        let record_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceIndexRecord,
        )
        .field_shape(WorthQueryEvidenceTag::new("owner"), owner.as_str())
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("reference"),
            reference_digest.evidence_identity(),
        )
        .seal()
        .into();
        Self {
            owner,
            family,
            reference_digest,
            record_identity,
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

    pub fn record_digest(&self) -> &str {
        self.record_identity.as_str()
    }

    pub fn record_identity(&self) -> &CausalEvidenceReferenceIndexRecordIdentity {
        &self.record_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalEvidenceReferenceIndexErrorKind {
    EvidenceOwnerMismatch,
}

impl CausalEvidenceReferenceIndexErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceOwnerMismatch => "evidence_owner_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceReferenceIndexError {
    kind: CausalEvidenceReferenceIndexErrorKind,
    family: CausalEvidenceFamily,
    supplied_owner: CausalEvidenceOwner,
    expected_owner: CausalEvidenceOwner,
    failure_identity: CausalEvidenceReferenceIndexErrorIdentity,
}

impl CausalEvidenceReferenceIndexError {
    fn new(
        kind: CausalEvidenceReferenceIndexErrorKind,
        family: CausalEvidenceFamily,
        supplied_owner: CausalEvidenceOwner,
        expected_owner: CausalEvidenceOwner,
    ) -> Self {
        let failure_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceIndexError,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("supplied_owner"),
            supplied_owner.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("expected_owner"),
            expected_owner.as_str(),
        )
        .seal()
        .into();
        Self {
            kind,
            family,
            supplied_owner,
            expected_owner,
            failure_identity,
        }
    }

    pub fn kind(&self) -> CausalEvidenceReferenceIndexErrorKind {
        self.kind
    }

    pub fn family(&self) -> CausalEvidenceFamily {
        self.family
    }

    pub fn supplied_owner(&self) -> CausalEvidenceOwner {
        self.supplied_owner
    }

    pub fn expected_owner(&self) -> CausalEvidenceOwner {
        self.expected_owner
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }
}

pub fn causal_evidence_reference_index(
    records: impl IntoIterator<Item = CausalEvidenceReferenceIndexRecord>,
) -> CausalEvidenceReferenceIndex {
    CausalEvidenceReferenceIndex::new(records.into_iter().collect())
}

pub fn causal_evidence_reference_index_record(
    owner: CausalEvidenceOwner,
    family: CausalEvidenceFamily,
    reference_digest: impl Into<CausalEvidenceReferenceInput>,
) -> Result<CausalEvidenceReferenceIndexRecord, CausalEvidenceReferenceIndexError> {
    let expected_owner = owner_for_family(family);
    if owner != expected_owner {
        return Err(CausalEvidenceReferenceIndexError::new(
            CausalEvidenceReferenceIndexErrorKind::EvidenceOwnerMismatch,
            family,
            owner,
            expected_owner,
        ));
    }
    let CausalEvidenceReferenceInput::Typed(reference_digest) = reference_digest.into();
    Ok(CausalEvidenceReferenceIndexRecord::new(
        owner,
        family,
        reference_digest,
    ))
}

pub(super) fn owner_for_family(family: CausalEvidenceFamily) -> CausalEvidenceOwner {
    match family {
        CausalEvidenceFamily::QueryInspection
        | CausalEvidenceFamily::QueryMutationCausality
        | CausalEvidenceFamily::QueryMutationProvenance
        | CausalEvidenceFamily::Policy
        | CausalEvidenceFamily::Redaction
        | CausalEvidenceFamily::Provenance => CausalEvidenceOwner::Query,
        CausalEvidenceFamily::RelationalAuthority | CausalEvidenceFamily::RelationalDecision => {
            CausalEvidenceOwner::Relational
        }
        CausalEvidenceFamily::SignalInvalidation
        | CausalEvidenceFamily::SignalEvaluation
        | CausalEvidenceFamily::SignalForensicAvailability
        | CausalEvidenceFamily::SignalReplayCursor
        | CausalEvidenceFamily::SignalLineage
        | CausalEvidenceFamily::SignalProvenance
        | CausalEvidenceFamily::Lineage => CausalEvidenceOwner::Signal,
        CausalEvidenceFamily::BridgeRoute
        | CausalEvidenceFamily::BridgeEvaluation
        | CausalEvidenceFamily::BridgeSourceMaterialization
        | CausalEvidenceFamily::BridgeSourceFailure
        | CausalEvidenceFamily::BridgeContinuity
        | CausalEvidenceFamily::BridgeMerge
        | CausalEvidenceFamily::BridgeStructural
        | CausalEvidenceFamily::BridgeStream
        | CausalEvidenceFamily::BridgePreview
        | CausalEvidenceFamily::BridgeWriteback
        | CausalEvidenceFamily::BridgeMapper
        | CausalEvidenceFamily::BridgeReplay => CausalEvidenceOwner::RuntimeBridge,
    }
}

fn index_identity(
    records: &BTreeMap<
        CausalEvidenceFamily,
        BTreeMap<CausalEvidenceReferenceDigest, CausalEvidenceReferenceIndexRecord>,
    >,
) -> CausalEvidenceReferenceIndexIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReferenceIndex)
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("records"),
            records
                .values()
                .flat_map(|family_records| family_records.values())
                .map(|record| record.record_identity().evidence_identity()),
        )
        .seal()
        .into()
}
