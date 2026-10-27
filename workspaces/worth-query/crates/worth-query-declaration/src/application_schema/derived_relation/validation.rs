//! Completed-schema closure for structural recipes and their descriptive consumers.
use std::collections::{BTreeMap, BTreeSet};

use crate::application_schema::identifier_validation::validate_simple_identifier;
use crate::application_schema::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

use super::{
    ApplicationDerivedRelationDefinitionDenial as Cause, ApplicationDerivedRelationNode,
    ApplicationDerivedRelationSourceKind, ErasedApplicationDerivedRelationDefinition,
};

pub(super) mod topology;

pub(in crate::application_schema) fn validate_closure(
    members: &[ApplicationSchemaMember],
    entities: &BTreeSet<&str>,
    stored_relations: &BTreeSet<(&str, &str, &str)>,
    operations: &BTreeSet<&str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut definitions = BTreeMap::new();
    for member in members {
        if let ApplicationSchemaMember::DerivedRelation { definition } = member {
            let name = definition.parts().relation.as_str();
            if definitions.insert(name, definition).is_some() {
                return Err(denial(name, Cause::DuplicateDefinition));
            }
        }
    }
    for (name, definition) in &definitions {
        let parts = definition.parts();
        for identifier in [&parts.relation, &parts.from, &parts.to] {
            validate_simple_identifier(identifier)?;
        }
        definition.validate().map_err(|cause| denial(name, cause))?;
        if !entities.contains(parts.from.as_str())
            || !entities.contains(parts.to.as_str())
            || !parts.laws.borrowed().minimum_kinds_are_valid(entities)
        {
            return Err(denial(name, Cause::MissingSource));
        }
        for node in &parts.expression.nodes {
            let ApplicationDerivedRelationNode::Source(source) = node else {
                continue;
            };
            for identifier in [&source.relation, &source.from, &source.to] {
                validate_simple_identifier(identifier)?;
            }
            let found = match source.kind {
                ApplicationDerivedRelationSourceKind::Stored => stored_relations.contains(&(
                    source.relation.as_str(),
                    source.from.as_str(),
                    source.to.as_str(),
                )),
                ApplicationDerivedRelationSourceKind::Derived => definitions
                    .get(source.relation.as_str())
                    .is_some_and(|definition| {
                        definition.parts().from == source.from && definition.parts().to == source.to
                    }),
            };
            if !found {
                return Err(denial(name, Cause::MissingSource));
            }
        }
    }
    topology::validate(&definitions)?;
    validate_consumers(members, &definitions, operations)
}

fn validate_consumers(
    members: &[ApplicationSchemaMember],
    definitions: &BTreeMap<&str, &ErasedApplicationDerivedRelationDefinition>,
    operations: &BTreeSet<&str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut collections = BTreeSet::new();
    for member in members {
        match member {
            ApplicationSchemaMember::RuleReadableCollection { association } => {
                let valid = definitions.contains_key(association.relation.as_str())
                    && collections.insert((&association.feature, &association.collection))
                    && association.validate().is_ok();
                if !valid {
                    return Err(denial(
                        &association.relation,
                        Cause::InvalidCollectionAssociation,
                    ));
                }
            }
            ApplicationSchemaMember::OperationDerivedRead {
                operation,
                relation,
            } => {
                validate_simple_identifier(operation)?;
                if !definitions.contains_key(relation.as_str()) {
                    return Err(denial(relation, Cause::MissingSource));
                }
                if !operations.contains(operation.as_str()) {
                    return Err(denial(relation, Cause::MissingOperation));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn denial(relation: &str, cause: Cause) -> ApplicationSchemaDeclarationDenial {
    ApplicationSchemaDeclarationDenial::InvalidApplicationDerivedRelation {
        relation: relation.to_owned(),
        cause,
    }
}

pub(in crate::application_schema) fn validate_member_identifiers(
    member: &ApplicationSchemaMember,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match member {
        ApplicationSchemaMember::DerivedRelation { definition } => definition
            .validate()
            .map_err(|cause| denial(&definition.parts().relation, cause)),
        ApplicationSchemaMember::RuleReadableCollection { association } => association
            .validate()
            .map_err(|cause| denial(&association.relation, cause)),
        ApplicationSchemaMember::OperationDerivedRead {
            operation,
            relation,
        } => {
            validate_simple_identifier(operation)?;
            validate_simple_identifier(relation)
        }
        _ => unreachable!("derived identifier router"),
    }
}
