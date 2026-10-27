//! One descriptive physical-leaf roster for installation and native lowering.
use super::{
    ApplicationDerivedRelationDefinitionDenial as Cause, ApplicationDerivedRelationNode as Node,
};
use super::{ApplicationDerivedRelationSource, ApplicationDerivedRelationSourceKind as Kind};
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use std::collections::{BTreeMap, BTreeSet};

/// Full descriptive source roster. Issuance still checks actual installed views.
pub struct ApplicationDerivedRelationSourceClosure<'schema> {
    /// True when any actual named definition needs the extended expression grammar.
    pub has_extended_expression: bool,
    pub physical: Vec<&'schema ApplicationDerivedRelationSource>,
    pub projected_domains:
        Vec<&'schema crate::application_schema::WorthQueryPortableProjectedMemberDomainParts>,
}

/// Resolve the complete physical closure of a named finite recipe. These
/// descriptions supply no installed binding, observation or read permission.
pub fn application_derived_relation_physical_sources<'schema>(
    members: &'schema [Member],
    relation: &str,
) -> Result<Vec<&'schema ApplicationDerivedRelationSource>, Denial> {
    Ok(application_derived_relation_source_closure(members, relation)?.physical)
}

/// Includes carrier access even when a lift produces no endpoint or physical leg.
pub fn application_derived_relation_source_closure<'schema>(
    members: &'schema [Member],
    relation: &str,
) -> Result<ApplicationDerivedRelationSourceClosure<'schema>, Denial> {
    let fail = |cause| Denial::InvalidApplicationDerivedRelation {
        relation: relation.to_owned(),
        cause,
    };
    let mut definitions = BTreeMap::new();
    let mut domains = BTreeMap::new();
    for member in members {
        if let Member::ProjectedMemberDomain { definition } = member {
            let parts = definition.parts();
            definition
                .validate()
                .map_err(|_| fail(Cause::ForeignCase))?;
            if domains.insert(parts.domain.as_str(), parts).is_some() {
                return Err(fail(Cause::ForeignCase));
            }
        }
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
    let mut projected = BTreeSet::new();
    let mut has_extended_expression = false;
    while let Some(name) = pending.pop() {
        if !visited.insert(name) {
            continue;
        }
        let definition = definitions
            .get(name)
            .ok_or_else(|| fail(Cause::MissingSource))?;
        has_extended_expression |= !definition.is_legacy_shape();
        for node in &definition.parts().expression.nodes {
            if let Node::CarrierLift {
                domain,
                carrier,
                case,
            } = node
            {
                let parts = domains
                    .get(domain.as_str())
                    .ok_or_else(|| fail(Cause::ForeignCase))?;
                if parts.carrier != *carrier || parts.cases.binary_search(case).is_err() {
                    return Err(fail(Cause::ForeignCase));
                }
                projected.insert(*parts);
            }
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
    Ok(ApplicationDerivedRelationSourceClosure {
        has_extended_expression,
        physical: leaves.into_iter().collect(),
        projected_domains: projected.into_iter().collect(),
    })
}
