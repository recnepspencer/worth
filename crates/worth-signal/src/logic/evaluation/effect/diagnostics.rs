use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::data::output::{
    ArtifactContinuityToken, CanonicalChangedRegions, ChangedRegion, OutputIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEnvelope {
    #[serde(default)]
    changed_regions: CanonicalChangedRegions,
    #[serde(default)]
    labels: SmallVec<[String; 2]>,
    #[serde(default)]
    output_identity: Option<OutputIdentity>,
    #[serde(default)]
    continuity_token: Option<ArtifactContinuityToken>,
}

impl DiagnosticEnvelope {
    pub fn from_parts(
        output_identity: Option<OutputIdentity>,
        continuity_token: Option<ArtifactContinuityToken>,
        changed_regions: Vec<ChangedRegion>,
        labels: Vec<String>,
    ) -> Option<Self> {
        let changed_regions = CanonicalChangedRegions::new(changed_regions);
        let labels = canonical_labels(labels);
        let envelope = Self {
            changed_regions,
            labels,
            output_identity,
            continuity_token,
        };

        if envelope.is_operationally_empty() {
            None
        } else {
            Some(envelope)
        }
    }

    pub fn changed_regions(&self) -> &[ChangedRegion] {
        self.changed_regions.as_slice()
    }

    pub fn labels(&self) -> &[String] {
        self.labels.as_slice()
    }

    pub fn output_identity(&self) -> Option<&OutputIdentity> {
        self.output_identity.as_ref()
    }

    pub fn continuity_token(&self) -> Option<&ArtifactContinuityToken> {
        self.continuity_token.as_ref()
    }

    fn is_operationally_empty(&self) -> bool {
        self.changed_regions.is_empty()
            && self.labels.is_empty()
            && self.output_identity.is_none()
            && self.continuity_token.is_none()
    }
}

fn canonical_labels(labels: Vec<String>) -> SmallVec<[String; 2]> {
    if labels.is_empty() {
        return SmallVec::new();
    }

    let mut canonical = SmallVec::from_vec(labels);
    if canonical.len() > 1 {
        canonical.sort_unstable();
        canonical.dedup();
    }
    canonical
}

impl crate::data::retained_storage::RetainedStorageMeasurement for DiagnosticEnvelope {
    fn retained_heap_charge(
        &self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        work.visit()?;
        self.changed_regions
            .retained_heap_charge(work)?
            .checked_add(self.labels.retained_heap_charge(work)?)?
            .checked_add(self.output_identity.retained_heap_charge(work)?)?
            .checked_add(self.continuity_token.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::logic::evaluation::DiagnosticEnvelope;

    #[test]
    fn operational_only_result_does_not_require_diagnostics() {
        assert!(DiagnosticEnvelope::from_parts(None, None, Vec::new(), Vec::new(),).is_none());
    }

    #[test]
    fn diagnostic_envelope_canonicalizes_labels_and_regions() {
        let envelope = DiagnosticEnvelope::from_parts(
            Some("artifact".into()),
            None,
            vec![
                crate::data::output::ChangedRegion::new("wing").with_detail("rib-2"),
                crate::data::output::ChangedRegion::new("wing").with_detail("rib-2"),
                crate::data::output::ChangedRegion::new("wing").with_detail("rib-1"),
            ],
            vec!["beta".into(), "alpha".into(), "alpha".into()],
        )
        .expect("diagnostics should be retained when identity data exists");

        assert_eq!(envelope.labels(), &["alpha".to_owned(), "beta".to_owned()]);
        assert_eq!(envelope.changed_regions().len(), 2);
        assert!(envelope.changed_regions()[0] < envelope.changed_regions()[1]);
    }
}
