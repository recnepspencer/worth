//! Fresh schema closure binds a finite domain to its actual stored carrier kind.
use super::{
    ApplicationProjectedMemberDomainDefinitionDenial as Cause,
    WorthQueryPortableProjectedMemberDomainParts,
};
use crate::application_schema::identifier_validation::validate_simple_identifier;
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember,
};

pub(in crate::application_schema) fn validate_member_identifiers(
    definition: &super::ErasedApplicationProjectedMemberDomainDefinition,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|cause| Denial::InvalidProjectedMemberDomain {
            domain: definition.parts().domain.clone(),
            cause,
        })
}
use std::collections::BTreeSet;

pub(super) fn validate_definition(
    parts: &WorthQueryPortableProjectedMemberDomainParts,
) -> Result<(), Cause> {
    if parts.semantic_version
        != super::ErasedApplicationProjectedMemberDomainDefinition::CURRENT_VERSION
    {
        return Err(Cause::UnsupportedVersion {
            observed: parts.semantic_version,
        });
    }
    for value in [&parts.domain, &parts.carrier]
        .into_iter()
        .chain(parts.cases.iter())
    {
        validate_simple_identifier(value).map_err(|_| Cause::InvalidIdentifier)?;
    }
    if parts.cases.is_empty() {
        return Err(Cause::EmptyCases);
    }
    if parts.cases.windows(2).any(|cases| cases[0] == cases[1]) {
        return Err(Cause::DuplicateCase);
    }
    Ok(())
}

pub(in crate::application_schema) fn validate_closure(
    members: &[ApplicationSchemaMember],
    entities: &BTreeSet<&str>,
) -> Result<(), Denial> {
    let mut domains = BTreeSet::new();
    for member in members {
        let ApplicationSchemaMember::ProjectedMemberDomain { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let reject = |cause| Denial::InvalidProjectedMemberDomain {
            domain: parts.domain.clone(),
            cause,
        };
        definition.validate().map_err(reject)?;
        if !domains.insert(parts.domain.as_str()) {
            return Err(reject(Cause::DuplicateDomain));
        }
        if entities.contains(parts.domain.as_str()) {
            return Err(reject(Cause::EntityNameCollision));
        }
        if !entities.contains(parts.carrier.as_str()) {
            return Err(reject(Cause::MissingCarrier));
        }
    }
    Ok(())
}
