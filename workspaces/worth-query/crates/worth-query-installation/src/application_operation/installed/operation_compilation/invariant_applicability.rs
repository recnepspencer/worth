use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;

use crate::domain_operation::{WorthQueryOperationTouchContract, WorthQueryOperationTouchScope};

pub(super) fn overlaps(
    touches: &WorthQueryOperationTouchContract,
    applicability: &[ApplicationInvariantScopeTarget],
) -> bool {
    touches.scopes().iter().any(|scope| {
        applicability
            .iter()
            .any(|target| scope_overlaps(scope, target))
    })
}

fn scope_overlaps(
    scope: &WorthQueryOperationTouchScope,
    target: &ApplicationInvariantScopeTarget,
) -> bool {
    match (scope, target) {
        (
            WorthQueryOperationTouchScope::CreateEntity(scope)
            | WorthQueryOperationTouchScope::DeleteEntity(scope),
            ApplicationInvariantScopeTarget::Entity(entity),
        ) => scope.entity() == entity,
        (
            WorthQueryOperationTouchScope::WriteField(scope),
            ApplicationInvariantScopeTarget::Entity(entity),
        ) => scope.entity() == entity,
        (
            WorthQueryOperationTouchScope::LinkRelation(scope)
            | WorthQueryOperationTouchScope::UnlinkRelation(scope),
            ApplicationInvariantScopeTarget::Relation(relation),
        ) => scope.relation() == relation,
        (
            WorthQueryOperationTouchScope::LinkRelation(scope)
            | WorthQueryOperationTouchScope::UnlinkRelation(scope),
            ApplicationInvariantScopeTarget::Entity(entity),
        ) => scope.from() == entity || scope.to() == entity,
        (WorthQueryOperationTouchScope::DeclaredDomain(_), _) => false,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::CanonicalDigestId;
    use worth_query_declaration::facade::application_schema::{
        ApplicationInvariantScopeTarget, ApplicationSchemaBindingIdentity,
    };

    use super::*;
    use crate::domain_operation::{
        WorthQueryOperationEntityTouchScope, WorthQueryOperationRelationTouchScope,
    };

    #[test]
    fn entity_mutation_requires_only_matching_entity_invariants() {
        let touches = contract(WorthQueryOperationTouchScope::CreateEntity(
            WorthQueryOperationEntityTouchScope::new(binding(), "Occurrence".to_owned()),
        ));

        assert!(overlaps(
            &touches,
            &[ApplicationInvariantScopeTarget::Entity(
                "Occurrence".to_owned()
            )]
        ));
        assert!(!overlaps(
            &touches,
            &[ApplicationInvariantScopeTarget::Entity("Body".to_owned())]
        ));
    }

    #[test]
    fn relation_mutation_requires_relation_and_endpoint_invariants() {
        let touches = contract(WorthQueryOperationTouchScope::LinkRelation(
            WorthQueryOperationRelationTouchScope::new(
                binding(),
                "Parent".to_owned(),
                "Occurrence".to_owned(),
                "Occurrence".to_owned(),
            ),
        ));

        for target in [
            ApplicationInvariantScopeTarget::Relation("Parent".to_owned()),
            ApplicationInvariantScopeTarget::Entity("Occurrence".to_owned()),
        ] {
            assert!(overlaps(&touches, &[target]));
        }
        assert!(!overlaps(
            &touches,
            &[ApplicationInvariantScopeTarget::Relation(
                "Membership".to_owned()
            )]
        ));
    }

    fn contract(scope: WorthQueryOperationTouchScope) -> WorthQueryOperationTouchContract {
        WorthQueryOperationTouchContract::Declared {
            graph_roles: Vec::new(),
            scopes: vec![scope],
        }
    }

    fn binding() -> ApplicationSchemaBindingIdentity {
        ApplicationSchemaBindingIdentity::from_installed_parts(
            1,
            1,
            CanonicalDigestId::new([1; 32]),
            CanonicalDigestId::new([2; 32]),
        )
    }
}
