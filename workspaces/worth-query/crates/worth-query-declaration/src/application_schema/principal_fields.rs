//! Principal selections carry expected codecs; composition checks actual field posture.
use super::{
    ApplicationFieldBindingRecipe, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

#[derive(Clone, Debug, Default)]
pub(super) struct PrincipalFieldExpectations {
    fields: Vec<(String, ApplicationFieldBindingRecipe, bool, bool)>,
}
impl PrincipalFieldExpectations {
    pub(super) fn expect(&mut self, binding: &str, fields: [&ApplicationFieldBindingRecipe; 3]) {
        for (field, writable, equality) in [
            (fields[0], false, true),
            (fields[1], true, false),
            (fields[2], false, true),
        ] {
            self.fields
                .push((binding.to_owned(), field.clone(), writable, equality));
        }
    }
    pub(super) fn validate(
        mut self,
        provenance: &super::ApplicationSchemaMemberProvenance,
        members: &[ApplicationSchemaMember],
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        for descriptor in provenance.mutation_bindings() {
            self.expect(
                descriptor.principal().name(),
                descriptor.principal().field_expectations(),
            );
        }
        for descriptor in provenance.query_bindings() {
            self.expect(
                descriptor.principal().name(),
                descriptor.principal().field_expectations(),
            );
        }
        let bindings = provenance.field_bindings();
        let catalog = &provenance.fragments.catalog;
        for (binding, expected, writable, equality) in self.fields {
            let locus = expected.locus();
            let codec_matches = bindings.iter().any(|actual| {
                actual.locus() == locus
                    && actual.binding_type() == expected.binding_type()
                    && actual.value_type() == expected.value_type()
            });
            let posture_matches = members.iter().any(|member| matches!(member,
                ApplicationSchemaMember::Field { entity, aspect, field,
                    writable: actual_write, equality_queryable, .. }
                if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                    && *actual_write == writable && (!equality || *equality_queryable)
            ));
            if catalog.internal_field(locus) || !codec_matches || !posture_matches {
                return Err(
                    ApplicationSchemaDeclarationDenial::InvalidPrincipalBindingField {
                        binding,
                        field: locus.clone(),
                    },
                );
            }
        }
        Ok(())
    }
}
