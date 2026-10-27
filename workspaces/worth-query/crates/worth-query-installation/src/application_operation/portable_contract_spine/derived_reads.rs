//! Declared derived reads require the real physical decision-read closure.
use super::*;
use worth_query_declaration::facade::application_schema::application_derived_relation_physical_sources;

pub(super) fn append(
    members: &[ApplicationSchemaMember],
    operation: &str,
    reads: &mut Vec<ApplicationOperationDecisionReadTarget>,
) -> SpineResult<()> {
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
        let sources =
            application_derived_relation_physical_sources(members, relation).map_err(|_| {
                WorthQueryPortableOperationContractSpineDenialKind::InvalidDerivedSource
            })?;
        for source in sources {
            reads.push(ApplicationOperationDecisionReadTarget::Entity {
                entity: source.from.clone(),
            });
            reads.push(ApplicationOperationDecisionReadTarget::Entity {
                entity: source.to.clone(),
            });
            reads.push(ApplicationOperationDecisionReadTarget::Relation {
                relation: source.relation.clone(),
                from: source.from.clone(),
                to: source.to.clone(),
            });
        }
    }
    Ok(())
}
