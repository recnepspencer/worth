//! A schema declares logical members without declaring a writable entity kind.
use super::ApplicationProjectedMemberDomainDefinition;
use crate::application_schema::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn projected_member_domain<Domain, Carrier>(
        self,
        definition: ApplicationProjectedMemberDomainDefinition<Schema, Domain, Carrier>,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::ProjectedMemberDomain {
            definition: definition.into_erased(),
        })
    }
}
