use crate::application_query::{
    ApplicationQueryDisclosureContract, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
};
use crate::application_resource::ApplicationResourceDisclosureRef;
use crate::application_schema::{ApplicationEncodedScalarValue, ApplicationScalarValueBinding};
use crate::portable_identity::WorthQueryPortableType;

impl ApplicationQueryDisclosureContract {
    pub fn disclose_resource_by<
        Resource: WorthQueryPortableType,
        Binding: ApplicationScalarValueBinding,
    >(
        mut self,
        selector: ApplicationResourceDisclosureRef<Resource>,
        disclosure_value: ApplicationEncodedScalarValue<Binding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Resource(selector.meaning()),
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
}
