//! One descriptive physical-leaf roster for installation and native lowering.
use super::{
    ApplicationDerivedRelationDefinitionDenial as Cause, ApplicationDerivedRelationNode as Node,
};
use super::{ApplicationDerivedRelationSource, ApplicationDerivedRelationSourceKind as Kind};
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use std::collections::{BTreeMap, BTreeSet};

/// Resolve the complete physical closure of a named finite recipe. These
/// descriptions supply no installed binding, observation or read permission.
pub fn application_derived_relation_physical_sources<'schema>(
    members: &'schema [Member],
    relation: &str,
) -> Result<Vec<&'schema ApplicationDerivedRelationSource>, Denial> {
    let fail = |cause| Denial::InvalidApplicationDerivedRelation {
        relation: relation.to_owned(),
        cause,
    };
    let mut definitions = BTreeMap::new();
    for member in members {
        if let Member::DerivedRelation { definition } = member {
            definition.validate().map_err(&fail)?;
            if definitions
                .insert(definition.parts().relation.as_str(), definition)
                .is_some()
            {
                return Err(fail(Cause::DuplicateDefinition));
            }
        }
    }
    // Use the same DAG check as completed-schema validation, not a second
    // interpretation of whether named cycles are legal.
    super::validation::topology::validate(&definitions)?;
    let start = definitions
        .get(relation)
        .ok_or_else(|| fail(Cause::MissingSource))?;
    let mut pending = vec![start.parts().relation.as_str()];
    let mut visited = BTreeSet::new();
    let mut leaves = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !visited.insert(name) {
            continue;
        }
        let definition = definitions
            .get(name)
            .ok_or_else(|| fail(Cause::MissingSource))?;
        for node in &definition.parts().expression.nodes {
            if let Node::Source(source) = node {
                match source.kind {
                    Kind::Stored => {
                        leaves.insert(source);
                    }
                    Kind::Derived => pending.push(source.relation.as_str()),
                }
            }
        }
    }
    Ok(leaves.into_iter().collect())
}
