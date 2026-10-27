//! Declared derived reads require the real physical decision-read closure.
use super::*;
use worth_query_declaration::facade::application_schema::application_derived_relation_source_closure;

pub(super) fn append(
    members: &[ApplicationSchemaMember],
    operation: &str,
    reads: &mut Vec<ApplicationOperationDecisionReadTarget>,
) -> SpineResult<Vec<String>> {
    let mut composite = Vec::new();
    for member in members {
        let ApplicationSchemaMember::OperationDerivedRead {
            operation: declared,
            relation,
        } = member
        else {
            continue;
        };
        if declared != operation {
            continue;
        }
        let closure =
            application_derived_relation_source_closure(members, relation).map_err(|_| {
                WorthQueryPortableOperationContractSpineDenialKind::InvalidDerivedSource
            })?;
        let aggregate_consumer = members.iter().any(|member| match member {
            ApplicationSchemaMember::DerivedComponentSet { definition } => {
                definition.relation == *relation
            }
            ApplicationSchemaMember::DerivedRelation { definition } => {
                definition.parts().relation == *relation && definition.parts().laws.orbit
            }
            _ => false,
        });
        if closure.has_extended_expression || aggregate_consumer {
            composite.push(relation.clone());
        }
        for source in closure.physical {
            let from = source
                .from
                .entity()
                .ok_or(WorthQueryPortableOperationContractSpineDenialKind::InvalidDerivedSource)?
                .to_owned();
            let to = source
                .to
                .entity()
                .ok_or(WorthQueryPortableOperationContractSpineDenialKind::InvalidDerivedSource)?
                .to_owned();
            reads.push(ApplicationOperationDecisionReadTarget::Entity {
                entity: from.clone(),
            });
            reads.push(ApplicationOperationDecisionReadTarget::Entity { entity: to.clone() });
            reads.push(ApplicationOperationDecisionReadTarget::Relation {
                relation: source.relation.clone(),
                from,
                to,
            });
        }
        for domain in closure.projected_domains {
            reads.push(ApplicationOperationDecisionReadTarget::Entity {
                entity: domain.carrier.clone(),
            });
        }
    }
    composite.sort();
    composite.dedup();
    Ok(composite)
}
