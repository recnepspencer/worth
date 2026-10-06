//! Bank-owned authority for one requested estate elevation.

use worth_foundational::facade::AspectValue;
use worth_query_host::facade::primary_graph::WorthQueryRequestedElevation;

/// Move-only Bank authority proving that the exact estate elevation was requested.
#[derive(Debug)]
pub struct BankRequestedEstateElevation {
    query: WorthQueryRequestedElevation,
}

impl BankRequestedEstateElevation {
    pub(in crate::estate_progression) const fn from_query(
        query: WorthQueryRequestedElevation,
    ) -> Self {
        Self { query }
    }

    pub(crate) const fn query(&self) -> &WorthQueryRequestedElevation {
        &self.query
    }

    pub(crate) fn into_query(self) -> WorthQueryRequestedElevation {
        self.query
    }

    pub const fn capability_identity(&self) -> [u8; 32] {
        self.query.capability_identity()
    }

    pub fn capability_authority_identity(&self) -> &str {
        self.query.capability_authority_identity()
    }

    pub const fn action(&self) -> &AspectValue {
        self.query.action()
    }

    pub const fn purpose(&self) -> &AspectValue {
        self.query.purpose()
    }

    pub const fn field(&self) -> Option<&AspectValue> {
        self.query.field()
    }

    pub const fn magnitude(&self) -> Option<&AspectValue> {
        self.query.magnitude()
    }

    pub const fn cardinality(&self) -> u32 {
        self.query.cardinality()
    }

    pub fn elevation_key(&self) -> &str {
        self.query.elevation_key()
    }

    pub const fn elevation_identity(&self) -> &AspectValue {
        self.query.elevation_identity()
    }

    pub const fn reason(&self) -> &AspectValue {
        self.query.reason()
    }

    pub const fn requested_status(&self) -> &AspectValue {
        self.query.requested_status()
    }

    pub const fn issued_at(&self) -> &AspectValue {
        self.query.issued_at()
    }

    pub const fn expires_at(&self) -> &AspectValue {
        self.query.expires_at()
    }

    pub fn review_key(&self) -> &str {
        self.query.review_key()
    }

    pub const fn review_identity(&self) -> &AspectValue {
        self.query.review_identity()
    }

    pub const fn review_status(&self) -> &AspectValue {
        self.query.review_status()
    }

    pub fn request_changed_record_count(&self) -> usize {
        self.query.publication_source().changed_record_count()
    }

    pub fn request_emitted_effect_count(&self) -> usize {
        self.query.publication_source().emitted_effect_count()
    }

    pub const fn request_canonical_work(&self) -> crate::BankCommitCanonicalWorkPhases {
        crate::BankCommitCanonicalWorkPhases::from_query(self.query.canonical_work())
    }
}
