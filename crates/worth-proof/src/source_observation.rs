use std::sync::Arc;

/// Runtime owner allowed to admit observations from one exact truth source.
///
/// A consumer may create its own owner, but evidence from that owner cannot
/// satisfy a service bound to a different occurrence.
#[derive(Debug)]
pub struct ConditionalSourceObservationOwner {
    occurrence: Arc<()>,
}

/// Cloneable authority bound into a runtime service before source evidence is
/// accepted. Its occurrence cannot be reconstructed from descriptive text.
#[derive(Clone, Debug)]
pub struct ConditionalSourceObservationAuthority {
    occurrence: Arc<()>,
}

/// Concrete proof that the owning runtime admitted one source observation.
/// Construction requires the exact owner whose authority is bound downstream.
#[derive(Debug)]
pub struct AdmittedConditionalSourceObservation {
    occurrence: Arc<()>,
    projection: Arc<str>,
}

/// Typed source posture presented to a conditional-evaluation owner.
///
/// The source-free variant carries no caller-selected identity. The consuming
/// owner must prove from the installed contract that no relational source is
/// declared before it admits that posture.
#[derive(Debug)]
pub enum ConditionalEvaluationSource {
    NoRelationalSource,
    AdmittedRelationalSource(AdmittedConditionalSourceObservation),
}

impl ConditionalSourceObservationOwner {
    pub fn fresh() -> Self {
        Self {
            occurrence: Arc::new(()),
        }
    }

    pub fn authority(&self) -> ConditionalSourceObservationAuthority {
        ConditionalSourceObservationAuthority {
            occurrence: Arc::clone(&self.occurrence),
        }
    }

    pub fn admit(&self, projection: impl Into<Arc<str>>) -> AdmittedConditionalSourceObservation {
        AdmittedConditionalSourceObservation {
            occurrence: Arc::clone(&self.occurrence),
            projection: projection.into(),
        }
    }
}

impl ConditionalSourceObservationAuthority {
    pub fn admits(&self, evidence: &AdmittedConditionalSourceObservation) -> bool {
        Arc::ptr_eq(&self.occurrence, &evidence.occurrence)
    }
}

impl AdmittedConditionalSourceObservation {
    pub fn projection(&self) -> &str {
        &self.projection
    }

    /// Conservative retained representation for a downstream custody ledger.
    pub fn retained_representation_bytes(&self) -> usize {
        let arc_headers = 4_usize.saturating_mul(std::mem::size_of::<usize>());
        arc_headers.saturating_add(self.projection.len())
    }
}

impl From<AdmittedConditionalSourceObservation> for ConditionalEvaluationSource {
    fn from(source: AdmittedConditionalSourceObservation) -> Self {
        Self::AdmittedRelationalSource(source)
    }
}

impl Default for ConditionalSourceObservationOwner {
    fn default() -> Self {
        Self::fresh()
    }
}

#[cfg(test)]
mod tests {
    use super::ConditionalSourceObservationOwner;

    #[test]
    fn evidence_retains_exact_owner_occurrence() {
        let owner = ConditionalSourceObservationOwner::fresh();
        let foreign = ConditionalSourceObservationOwner::fresh();
        let evidence = owner.admit("snapshot-a");

        assert!(owner.authority().admits(&evidence));
        assert!(!foreign.authority().admits(&evidence));
        assert_eq!(evidence.projection(), "snapshot-a");
    }
}
