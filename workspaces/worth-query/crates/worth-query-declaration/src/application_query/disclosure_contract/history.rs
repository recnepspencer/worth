use crate::application_query::{
    ApplicationHistoryMetadataRef, ApplicationQueryDisclosureContract,
    ApplicationQueryDisclosureRule, ApplicationQueryDisclosureSelector,
    ApplicationQueryInfluenceContract,
};
use crate::application_schema::{ApplicationEncodedScalarValue, ApplicationScalarValueBinding};
use crate::portable_identity::WorthQueryPortableType;

impl ApplicationQueryDisclosureContract {
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
