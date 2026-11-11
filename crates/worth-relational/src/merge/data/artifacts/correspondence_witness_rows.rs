use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::merge::data::{
    IdentityBasisKind, IdentityBasisScope, IdentityMatchCandidate, IdentityMatchClass,
    IdentityResolutionReason, MergeRecordIdentity,
};
use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeCorrespondenceWitnessPosture {
    Admitted,
    DeniedAmbiguous,
    UnavailableMissingTarget,
    DeniedSchemaNonUniqueSource,
    DeniedSchemaNonUniqueTarget,
    DeniedSchemaNonUniqueSourceAndTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeCorrespondenceWitnessRow {
    scope: Option<IdentityBasisScope>,
    source_record: RecordRef,
    target_record: Option<RecordRef>,
    source: MergeRecordIdentity,
    target: Option<MergeRecordIdentity>,
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
    authority_basis: IdentityBasisKind,
    candidate_digest: String,
    posture: RelationalMergeCorrespondenceWitnessPosture,
}

impl RelationalMergeCorrespondenceWitnessRow {
    pub(crate) fn from_candidate(
        candidate: &IdentityMatchCandidate,
        posture: RelationalMergeCorrespondenceWitnessPosture,
    ) -> Self {
        Self {
            scope: candidate.scope.clone(),
            source_record: candidate.source_record.clone(),
            target_record: candidate.target_record.clone(),
            source: candidate.source.clone(),
            target: candidate.target.clone(),
            match_class: candidate.match_class.clone(),
            reason: candidate.reason.clone(),
            authority_basis: candidate.basis.clone(),
            candidate_digest: identity_match_candidate_digest(candidate),
            posture,
        }
    }

    pub fn scope(&self) -> Option<&IdentityBasisScope> {
        self.scope.as_ref()
    }

    pub fn source_record(&self) -> &RecordRef {
        &self.source_record
    }

    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }

    pub fn source(&self) -> &MergeRecordIdentity {
        &self.source
    }

    pub fn target(&self) -> Option<&MergeRecordIdentity> {
        self.target.as_ref()
    }

    pub fn match_class(&self) -> &IdentityMatchClass {
        &self.match_class
    }

    pub fn reason(&self) -> &IdentityResolutionReason {
        &self.reason
    }

    pub fn authority_basis(&self) -> &IdentityBasisKind {
        &self.authority_basis
    }

    pub fn candidate_digest(&self) -> &str {
        &self.candidate_digest
    }

    pub fn posture(&self) -> RelationalMergeCorrespondenceWitnessPosture {
        self.posture
    }

    pub(crate) fn candidate(&self) -> IdentityMatchCandidate {
        IdentityMatchCandidate {
            scope: self.scope.clone(),
            source_record: self.source_record.clone(),
            target_record: self.target_record.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
            match_class: self.match_class.clone(),
            reason: self.reason.clone(),
            basis: self.authority_basis.clone(),
        }
    }

    pub(crate) fn expected_candidate_digest(&self) -> String {
        identity_match_candidate_digest(&self.candidate())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeCorrespondenceWitnessRowWire {
    scope: Option<IdentityBasisScope>,
    source_record: RecordRef,
    target_record: Option<RecordRef>,
    source: MergeRecordIdentity,
    target: Option<MergeRecordIdentity>,
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
    authority_basis: IdentityBasisKind,
    candidate_digest: String,
    posture: RelationalMergeCorrespondenceWitnessPosture,
}

impl<'de> Deserialize<'de> for RelationalMergeCorrespondenceWitnessRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeCorrespondenceWitnessRowWire::deserialize(deserializer)?;
        let row = Self {
            scope: wire.scope,
            source_record: wire.source_record,
            target_record: wire.target_record,
            source: wire.source,
            target: wire.target,
            match_class: wire.match_class,
            reason: wire.reason,
            authority_basis: wire.authority_basis,
            candidate_digest: wire.candidate_digest,
            posture: wire.posture,
        };
        if row.candidate_digest != row.expected_candidate_digest() {
            return Err(D::Error::custom(
                "merge correspondence witness row candidate digest does not match retained candidate truth",
            ));
        }
        Ok(row)
    }
}

pub(crate) fn identity_match_candidate_digest(candidate: &IdentityMatchCandidate) -> String {
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.merge.identity_match_candidate.v1");
    {
        let mut writer = super::correspondence_witness::CorrespondenceHashWriter(&mut digest);
        rmp_serde::encode::write_named(&mut writer, candidate)
            .expect("closed native identity candidate must encode");
    }
    super::correspondence_witness::lowercase_sha256(digest.finalize().into())
}
