//! Exact indexed-read causes remain attached to their owning attempt refusal.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind,
};

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn indexed_lookup_denied(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: &str,
        denial: BoundedEntityFieldLookupDenial,
    ) -> Self {
        if let BoundedEntityFieldLookupDenialKind::Execution(cause) = denial.kind() {
            return Self::relational_read_stopped(cause, subject);
        }
        Self::from_cause(
            kind,
            subject.to_owned(),
            AttemptDenialCause::IndexedLookup(Box::new(denial)),
        )
    }

    /// The original exact-index refusal, including its installed index identity.
    pub fn indexed_lookup_denial(&self) -> Option<&BoundedEntityFieldLookupDenial> {
        match self.cause() {
            Some(AttemptDenialCause::IndexedLookup(denial)) => Some(denial),
            _ => None,
        }
    }
}
