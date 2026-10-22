//! Static feature membership before its canonical declaration is erased.
use crate::application_program::{ApplicationFeatureSpec, ApplicationProgramOutputRootsShape};
use crate::application_schema::{ApplicationSchema, ApplicationSchemaDeclarationBuilder};

/// A selected feature definition projects its authored members, not a runtime registry.
pub trait ApplicationFeatureDefinition<S: ApplicationSchema> {
    type Outputs: ApplicationProgramOutputRootsShape<S>;
    fn declare_members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S>;
    fn feature_specs() -> Vec<ApplicationFeatureSpec>;
}

impl<S, A, B> ApplicationFeatureDefinition<S> for (A, B)
where
    S: ApplicationSchema,
    A: ApplicationFeatureDefinition<S>,
    B: ApplicationFeatureDefinition<S>,
{
    type Outputs = (A::Outputs, B::Outputs);
    fn declare_members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        B::declare_members(A::declare_members(schema))
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = A::feature_specs();
        specs.extend(B::feature_specs());
        specs
    }
}
