use std::sync::Arc;
use worth_foundational::facade::admit_foundational_authority_identity;
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
};

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity_authority::{
    query_receipt_admission_authority, QueryEntityIdentityKind, QueryReceiptAuthorityIdentity,
};

#[derive(Clone)]
pub struct WorthQueryEntityIdentity {
    inner: WorthQueryEntityIdentityInner,
    _authority: Option<QueryReceiptAuthorityIdentity<Arc<str>, QueryEntityIdentityKind>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorthQueryEntityIdentityInner {
    RelationalRecord {
        relational_record: RelationalBridgeRecordIdentityParts,
    },
    Preview {
        evidence_identity: WorthQueryEvidenceIdentity,
    },
}

impl WorthQueryEntityIdentity {
    pub fn from_bridge_record_projection(
        relational_record: RelationalBridgeRecordIdentityParts,
    ) -> Self {
        Self {
            inner: WorthQueryEntityIdentityInner::RelationalRecord { relational_record },
            _authority: None,
        }
    }

    pub(crate) fn from_relational_record(
        relational_record: RelationalBridgeRecordIdentityParts,
    ) -> Self {
        Self::from_bridge_record_projection(relational_record)
    }

    pub(crate) fn from_runtime_receipt_record(
        relational_record: RelationalBridgeRecordIdentityParts,
    ) -> Self {
        let authority = admit_foundational_authority_identity(
            Arc::<str>::from(relational_record.terminal_projection_for_reporting()),
            query_receipt_admission_authority(),
        );
        Self {
            inner: WorthQueryEntityIdentityInner::RelationalRecord { relational_record },
            _authority: Some(authority),
        }
    }

    pub(crate) fn preview(evidence_identity: WorthQueryEvidenceIdentity) -> Self {
        Self {
            inner: WorthQueryEntityIdentityInner::Preview { evidence_identity },
            _authority: None,
        }
    }

    pub fn admit_authored_entity_token(
        token: crate::identity_authority::QueryExternalIdentityToken<
            Arc<str>,
            QueryEntityIdentityKind,
        >,
    ) -> Self {
        super::super::truth_identity_admission::admit_authored_entity_token(token)
    }

    pub(crate) fn relational_record_parts(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        match &self.inner {
            WorthQueryEntityIdentityInner::RelationalRecord { relational_record } => {
                Some(*relational_record)
            }
            WorthQueryEntityIdentityInner::Preview { .. } => None,
        }
    }

    pub(crate) fn relational_entity_record_parts(
        &self,
    ) -> Option<RelationalBridgeRecordIdentityParts> {
        self.relational_record_parts()
            .filter(|parts| parts.kind() == RelationalBridgeRecordIdentityKind::Entity)
    }

    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        match &self.inner {
            WorthQueryEntityIdentityInner::RelationalRecord { relational_record } => {
                entity_evidence_from_relational_record(*relational_record)
            }
            WorthQueryEntityIdentityInner::Preview { evidence_identity } => {
                evidence_identity.clone()
            }
        }
    }

    pub(crate) fn terminal_projection_for_reporting(&self) -> String {
        self.evidence_identity()
            .terminal_projection_for_reporting()
            .to_string()
    }

    pub fn is_same_current_identity_as(&self, candidate: &Self) -> bool {
        self._authority.is_some() && candidate._authority.is_some() && self.inner == candidate.inner
    }

    pub(crate) fn has_current_authority(&self) -> bool {
        self._authority.is_some()
    }

    pub(crate) fn admit_runtime_write_authority(mut self) -> Self {
        if self._authority.is_none() {
            let basis = self.evidence_identity();
            self._authority = Some(admit_foundational_authority_identity(
                Arc::<str>::from(basis.terminal_projection_for_reporting()),
                query_receipt_admission_authority(),
            ));
        }
        self
    }

    pub(crate) fn compare_for_candidate_index(&self, candidate: &Self) -> std::cmp::Ordering {
        self.evidence_identity()
            .compare_same_scheme(&candidate.evidence_identity())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl std::fmt::Debug for WorthQueryEntityIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let posture = if self._authority.is_some() {
            "current"
        } else {
            "projection"
        };
        formatter
            .debug_struct("WorthQueryEntityIdentity")
            .field("posture", &posture)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryEntityIdentity {
    fn eq(&self, candidate: &Self) -> bool {
        self.inner == candidate.inner
    }
}
impl Eq for WorthQueryEntityIdentity {}
impl PartialOrd for WorthQueryEntityIdentity {
    fn partial_cmp(&self, candidate: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(candidate))
    }
}
impl Ord for WorthQueryEntityIdentity {
    fn cmp(&self, candidate: &Self) -> std::cmp::Ordering {
        self.compare_for_candidate_index(candidate)
    }
}
impl std::hash::Hash for WorthQueryEntityIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.terminal_projection_for_reporting(), state);
    }
}

fn entity_evidence_from_relational_record(
    relational_record: RelationalBridgeRecordIdentityParts,
) -> WorthQueryEvidenceIdentity {
    let kind = match relational_record.kind() {
        RelationalBridgeRecordIdentityKind::Entity => "entity",
        RelationalBridgeRecordIdentityKind::Relation => "relation",
    };
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptEntityIdentity)
        .field_shape(WorthQueryEvidenceTag::new("record_kind"), kind)
        .field_usize(
            WorthQueryEvidenceTag::new("partition_id"),
            relational_record.partition_id() as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("local_slot"),
            relational_record.local_slot() as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("generation"),
            relational_record.generation() as usize,
        )
        .seal()
}
