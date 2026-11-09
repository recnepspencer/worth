use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::evidence_identities::slice_intent_identity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuerySubscriptionSliceKind {
    AuthorizedProjection,
    Membership,
    Ordering,
    Grouping,
    RelationScope,
    ViewShapeMetadata,
}

impl QuerySubscriptionSliceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorizedProjection => "authorized_projection",
            Self::Membership => "membership",
            Self::Ordering => "ordering",
            Self::Grouping => "grouping",
            Self::RelationScope => "relation_scope",
            Self::ViewShapeMetadata => "view_shape_metadata",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct QuerySubscriptionSlicePart {
    kind: QuerySubscriptionSliceKind,
    ordinal: usize,
}

impl QuerySubscriptionSlicePart {
    pub(super) fn new(kind: QuerySubscriptionSliceKind, ordinal: usize) -> Self {
        Self { kind, ordinal }
    }

    pub fn kind(&self) -> &QuerySubscriptionSliceKind {
        &self.kind
    }

    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub(super) fn canonical_part(&self) -> String {
        format!("{}:{}", self.kind.as_str(), self.ordinal)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSliceIntent {
    parts: Vec<QuerySubscriptionSlicePart>,
    slice_intent_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionSliceIntent {
    pub(super) fn from_canonical_parts(
        mut parts: Vec<QuerySubscriptionSlicePart>,
    ) -> Result<Self, IdentityDerivationDenial> {
        parts.sort();
        parts.dedup();
        let slice_intent_identity = slice_intent_identity(&parts)?;
        Ok(Self {
            parts,
            slice_intent_identity,
        })
    }

    pub fn parts(&self) -> &[QuerySubscriptionSlicePart] {
        &self.parts
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.slice_intent_identity
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}
