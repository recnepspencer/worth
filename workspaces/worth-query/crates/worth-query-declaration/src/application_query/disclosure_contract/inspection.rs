use crate::application_query::{
    ApplicationQueryBinding, ApplicationQueryDisclosureContract, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
    ApplicationQueryInspectionMetadataRef,
};
use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationScalarValueBinding, ApplicationSchema,
};

impl ApplicationQueryDisclosureContract {
    /// Author one exact binding's independent inspection metadata facet.
    pub fn disclose_inspection_metadata_by<Schema, QueryBinding, ValueBinding>(
        mut self,
        selector: ApplicationQueryInspectionMetadataRef<Schema, QueryBinding>,
        disclosure_value: ApplicationEncodedScalarValue<ValueBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Schema: ApplicationSchema,
        QueryBinding: ApplicationQueryBinding<Schema>,
        ValueBinding: ApplicationScalarValueBinding,
    {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::InspectionMetadata(selector.meaning()),
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
}
