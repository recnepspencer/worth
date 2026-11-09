//! Correction selections are compiler inputs, never published native binding inventory.
use super::{ApplicationFieldBindingRecipe, ApplicationSchemaDeclarationDenial};

#[derive(Clone, Debug, Default)]
pub(super) struct CorrectionFieldExpectations {
    fields: Vec<(String, ApplicationFieldBindingRecipe)>,
    refusal: Option<ApplicationSchemaDeclarationDenial>,
}
impl CorrectionFieldExpectations {
    pub(super) fn expect(&mut self, fields: Vec<(String, ApplicationFieldBindingRecipe)>) {
        self.fields.extend(fields);
    }
    pub(super) fn refuse(&mut self, denial: ApplicationSchemaDeclarationDenial) {
        if self.refusal.is_none() {
            self.refusal = Some(denial);
        }
    }
    pub(super) fn validate(
        self,
        catalog: &super::fragment::FragmentCatalog,
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        if let Some(denial) = self.refusal {
            return Err(denial);
        }
        for (correction, expected) in self.fields {
            if !catalog.declared_field_matches(&expected) {
                return Err(ApplicationSchemaDeclarationDenial::InvalidCorrectionField {
                    correction,
                    field: expected.locus().clone(),
                });
            }
        }
        Ok(())
    }
}
