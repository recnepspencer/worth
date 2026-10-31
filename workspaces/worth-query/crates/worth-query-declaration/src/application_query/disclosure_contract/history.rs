use crate::application_query::{
    ApplicationHistoryMetadataRef, ApplicationQueryDisclosureContract,
    ApplicationQueryDisclosureRule, ApplicationQueryDisclosureSelector,
    ApplicationQueryInfluenceContract,
};
use crate::application_schema::{ApplicationEncodedScalarValue, ApplicationScalarValueBinding};
use crate::portable_identity::WorthQueryPortableType;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryMetadataDisclosureDenial;

impl ApplicationQueryDisclosureContract {
    /// Metadata rules must cover exactly this history's closed facet set.
    pub fn validate_history_metadata(
        &self,
        history: &WorthQueryPortableTypeIdentity,
    ) -> Result<(), ApplicationHistoryMetadataDisclosureDenial> {
        use crate::application_query::{
            ApplicationHistoryMetadataFacet, ApplicationQueryDisclosurePosture,
        };
        let refused = ApplicationHistoryMetadataDisclosureDenial;
        let valid_name = |value: &str| {
            !value.is_empty()
                && !value
                    .chars()
                    .any(|ch| ch.is_whitespace() || ch.is_control())
        };
        if self.classification().trim().is_empty() || !history.is_valid() {
            return Err(refused);
        }
        match self.posture() {
            ApplicationQueryDisclosurePosture::Public
                if self.rules().is_empty()
                    && self.capability_name().is_none()
                    && self.capability_type().is_none() =>
            {
                return Ok(())
            }
            ApplicationQueryDisclosurePosture::Governed
                if self.capability_name().is_some_and(valid_name)
                    && self
                        .capability_identity()
                        .is_some_and(|identity| identity.is_valid()) => {}
            _ => return Err(refused),
        }
        let mut seen = std::collections::BTreeSet::new();
        for rule in self.rules() {
            let ApplicationQueryDisclosureSelector::HistoryMetadata(metadata) = rule.selector()
            else {
                return Err(refused);
            };
            if metadata.history_identity() != *history
                || !rule.influence().permits(metadata.facet().influence())
                || !seen.insert(metadata.facet())
            {
                return Err(refused);
            }
        }
        if seen.len() != ApplicationHistoryMetadataFacet::ALL.len() {
            return Err(refused);
        }
        Ok(())
    }
    /// Disclose one explicit history metadata facet. Payload field disclosure
    /// and SubjectRead's ordinary result slots cannot substitute for this rule.
    pub fn disclose_history_metadata_by<History, Binding>(
        mut self,
        selector: ApplicationHistoryMetadataRef<History>,
        disclosure_value: ApplicationEncodedScalarValue<Binding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        History: WorthQueryPortableType,
        Binding: ApplicationScalarValueBinding,
    {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::HistoryMetadata(selector.meaning()),
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
}
