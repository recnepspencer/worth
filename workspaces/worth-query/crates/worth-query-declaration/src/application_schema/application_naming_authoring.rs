use super::{
    ApplicationSchema, ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
    ApplicationSchemaMember,
};
use crate::application_naming::{ApplicationGroupNameScheme, ApplicationNameScheme};

impl<S: ApplicationSchema> ApplicationSchemaDeclarationBuilder<S> {
    /// Registers a declared persistent name scheme on this schema declaration.
    ///
    /// Installation validates role-order acyclicity, split-discriminator
    /// presence, subject closure, and scheme uniqueness before runtime starts.
    pub fn application_name_scheme<K>(
        self,
        scheme: ApplicationNameScheme<S, K>,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial> {
        Ok(
            self.push_member(ApplicationSchemaMember::ApplicationNameScheme {
                definition: scheme.into_erased(),
            }),
        )
    }

    /// Registers a declared persistent group name scheme (with its continuity
    /// rule and anchor/boundary roles) on this schema declaration.
    pub fn application_group_name_scheme<G>(
        self,
        scheme: ApplicationGroupNameScheme<S, G>,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial> {
        Ok(
            self.push_member(ApplicationSchemaMember::ApplicationGroupNameScheme {
                definition: scheme.into_erased(),
            }),
        )
    }
}
