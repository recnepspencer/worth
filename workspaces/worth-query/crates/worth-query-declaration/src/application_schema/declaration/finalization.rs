use std::marker::PhantomData;

use super::{
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ErasedApplicationSchemaDeclaration,
};
use crate::application_schema::canonical_identity::{
    canonical_identity, ApplicationSchemaCanonicalHeader,
};
use crate::application_schema::contribution::{
    lower_authored_contributions, AuthoredApplicationSchemaContribution,
};
use crate::application_schema::identifier_validation::{
    validate_member_identifiers, validate_schema_header,
};
use crate::application_schema::member_closure::validate_member_closure;
use crate::application_schema::operation_contract_cardinality::validate_operation_contract_cardinality;
use crate::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaDeclarationDenial,
    ApplicationSchemaMember,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn build(
        mut self,
    ) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
        if let Some(denial) = self.refusal {
            return Err(denial);
        }
        crate::application_schema::fragment::expand_record_access(
            &self.member_provenance.fragments.catalog,
            &self.member_provenance.fragments.access,
            &mut self.members,
            &mut self.contributions,
        )?;
        validate_schema_header(self.owner, self.name)?;
        validate_member_identifiers(&self.members)?;
        crate::application_schema::relation_integrity::validate_relation_integrity(&self.members)?;
        crate::application_schema::member_identity_uniqueness::validate_member_identity_uniqueness(
            &self.members,
        )?;
        validate_operation_contract_cardinality(&self.members)?;
        self.members.sort();
        self.member_provenance.normalize();
        self.mutation_references.validate(&self.member_provenance)?;
        self.correction_fields
            .validate(&self.member_provenance.fragments.catalog)?;
        self.principal_fields
            .validate(&self.member_provenance, &self.members)?;
        self.member_provenance
            .fragments
            .validate(self.member_provenance.field_bindings())?;
        if self.member_provenance.has_conflicting_field_binding()
            || !self.member_provenance.field_bindings_match(&self.members)
        {
            return Err(ApplicationSchemaDeclarationDenial::ConflictingFieldBinding);
        }
        if let Some(pair) = self.members.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(
                if matches!(pair[0], ApplicationSchemaMember::ContentFormat { .. }) {
                    ApplicationSchemaDeclarationDenial::DuplicateContentFormat
                } else {
                    ApplicationSchemaDeclarationDenial::DuplicateMember
                },
            );
        }
        expand_shared_owner_relation_reads(&mut self.members, &mut self.contributions);
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

/// Emits one canonical governing-relation decision read for each operation's
/// `SharedOwnerManagedIndex` reads when not already explicitly declared.
fn expand_shared_owner_relation_reads(
    members: &mut Vec<ApplicationSchemaMember>,
    contributions: &mut [AuthoredApplicationSchemaContribution],
) {
    let mut implied = Vec::new();
    for source in members.iter() {
        let ApplicationSchemaMember::OperationDecisionRead {
            operation,
            target:
                ApplicationOperationDecisionReadTarget::SharedOwnerManagedIndex {
                    relation,
                    owner,
                    member,
                    ..
                },
        } = source
        else {
            continue;
        };
        let governing = ApplicationSchemaMember::OperationDecisionRead {
            operation: operation.clone(),
            target: ApplicationOperationDecisionReadTarget::Relation {
                relation: relation.clone(),
                from: owner.clone(),
                to: member.clone(),
            },
        };
        if members.binary_search(&governing).is_ok() {
            continue;
        }
        for contribution in contributions.iter_mut() {
            contribution.expand_member(source, &governing);
        }
        implied.push(governing);
    }
    if !implied.is_empty() {
        implied.sort();
        implied.dedup();
        members.extend(implied);
        members.sort();
    }
}
