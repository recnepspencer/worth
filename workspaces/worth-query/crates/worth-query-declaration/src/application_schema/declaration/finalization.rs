use std::marker::PhantomData;

use super::{
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ErasedApplicationSchemaDeclaration,
};
use crate::application_schema::canonical_identity::{
    canonical_identity, ApplicationSchemaCanonicalHeader,
};
use crate::application_schema::contribution::lower_authored_contributions;
use crate::application_schema::identifier_validation::{
    validate_member_identifiers, validate_schema_header,
};
use crate::application_schema::member_closure::validate_member_closure;
use crate::application_schema::operation_contract_cardinality::validate_operation_contract_cardinality;
use crate::application_schema::ApplicationSchemaDeclarationDenial;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn build(
        mut self,
    ) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
        validate_schema_header(self.owner, self.name)?;
        validate_member_identifiers(&self.members)?;
        crate::application_schema::relation_integrity::validate_relation_integrity(&self.members)?;
        crate::application_schema::member_identity_uniqueness::validate_member_identity_uniqueness(
            &self.members,
        )?;
        validate_operation_contract_cardinality(&self.members)?;
        self.members.sort();
        self.member_provenance.normalize();
        if self.member_provenance.has_conflicting_field_binding()
            || !self.member_provenance.field_bindings_match(&self.members)
        {
            return Err(ApplicationSchemaDeclarationDenial::ConflictingFieldBinding);
        }
        if self.members.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
        }
        let contributions = lower_authored_contributions(self.contributions, &self.members)?;
        validate_member_closure(&self.members)?;
        let identity = canonical_identity(
            ApplicationSchemaCanonicalHeader {
                owner: self.owner,
                name: self.name,
                major: self.major,
                minor: self.minor,
            },
            &self.members,
            &contributions,
        );
        Ok(ApplicationSchemaDeclaration {
            erased: ErasedApplicationSchemaDeclaration {
                owner: self.owner.to_string(),
                name: self.name.to_string(),
                major: self.major,
                minor: self.minor,
                identity,
                members: self.members,
                contributions,
            },
            member_provenance: self.member_provenance,
            _schema: PhantomData,
        })
    }
}
