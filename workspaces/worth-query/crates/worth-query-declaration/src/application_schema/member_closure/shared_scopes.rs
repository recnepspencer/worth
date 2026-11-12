//! Shared contents close once; every operation reference must resolve exactly.
use super::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member, ClosureIndex,
    Locus,
};

pub(super) fn validate(index: &ClosureIndex<'_>) -> Result<(), Denial> {
    for member in index.members {
        match member {
            Member::DecisionReadScope { definition } => {
                for target in definition.targets() {
                    if !index.decision_read_target_exists(target) {
                        return Err(Denial::MissingOperationDecisionReadDependency {
                            operation: definition.identity().to_owned(),
                            dependency: Locus::DecisionReadTarget(target.clone()),
                        });
                    }
                }
                for relation in definition.derived_relations() {
                    if !index.members.iter().any(|member| matches!(member, Member::DerivedRelation { definition } if definition.parts().relation == *relation)) {
                        return Err(Denial::InvalidApplicationDerivedRelation { relation: relation.clone(), cause: crate::application_schema::ApplicationDerivedRelationDefinitionDenial::MissingSource });
                    }
                }
            }
            Member::OperationDecisionReadScope { operation, scope }
                if !index.operations.contains(operation.as_str())
                    || !index.members.iter().any(|member| {
                        matches!(member,
                    Member::DecisionReadScope { definition } if definition.identity() == scope)
                    }) =>
            {
                return Err(Denial::MissingDecisionReadScope {
                    operation: operation.clone(),
                    scope: scope.clone(),
                });
            }
            _ => {}
        }
    }
    Ok(())
}
