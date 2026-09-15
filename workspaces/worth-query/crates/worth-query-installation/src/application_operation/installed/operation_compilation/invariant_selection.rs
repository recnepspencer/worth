use worth_query_declaration::facade::application_schema::ApplicationInvariantGroup;

use crate::domain_operation::{WorthQueryOperationTouchContract, WorthQueryOperationTouchScope};

pub(super) fn requires(
    touches: &WorthQueryOperationTouchContract,
    required_groups: &[ApplicationInvariantGroup],
) -> bool {
    touches.scopes().iter().any(|scope| {
        required_groups
            .iter()
            .any(|group| invalidates(scope, *group))
    })
}

fn invalidates(scope: &WorthQueryOperationTouchScope, group: ApplicationInvariantGroup) -> bool {
    use ApplicationInvariantGroup as Group;
    match scope {
        WorthQueryOperationTouchScope::CreateEntity(_) => matches!(
            group,
            Group::StorageCoherence
                | Group::IdentityCoherence
                | Group::SchemaCompliance
                | Group::PublicationCoherence
                | Group::VersionVisibility
        ),
        WorthQueryOperationTouchScope::DeleteEntity(_) => matches!(
            group,
            Group::AdjacencyIntegrity
                | Group::StorageCoherence
                | Group::LineageIntegrity
                | Group::RelationIntegrity
                | Group::PublicationCoherence
                | Group::VersionVisibility
        ),
        WorthQueryOperationTouchScope::WriteField(_) => {
            matches!(group, Group::IdentityCoherence | Group::SchemaCompliance)
        }
        WorthQueryOperationTouchScope::LinkRelation(_) => matches!(
            group,
            Group::AdjacencyIntegrity
                | Group::StorageCoherence
                | Group::SchemaCompliance
                | Group::RelationIntegrity
                | Group::PublicationCoherence
                | Group::VersionVisibility
        ),
        WorthQueryOperationTouchScope::UnlinkRelation(_) => matches!(
            group,
            Group::AdjacencyIntegrity
                | Group::StorageCoherence
                | Group::RelationIntegrity
                | Group::PublicationCoherence
                | Group::VersionVisibility
        ),
        WorthQueryOperationTouchScope::DeclaredDomain(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::CanonicalDigestId;
    use worth_query_declaration::facade::application_schema::{
        ApplicationInvariantGroup, ApplicationSchemaBindingIdentity,
    };

    use super::*;
    use crate::domain_operation::{
        WorthQueryOperationEntityTouchScope, WorthQueryOperationRelationTouchScope,
    };

    #[test]
    fn entity_create_requires_schema_but_not_relation_invariants() {
        let touches = contract(WorthQueryOperationTouchScope::CreateEntity(
            WorthQueryOperationEntityTouchScope::new(binding(), "Occurrence".to_owned()),
        ));

        assert!(requires(
            &touches,
            &[ApplicationInvariantGroup::SchemaCompliance]
        ));
        assert!(!requires(
            &touches,
            &[ApplicationInvariantGroup::RelationIntegrity]
        ));
    }

    #[test]
    fn relation_create_requires_relation_and_schema_invariants() {
        let touches = contract(WorthQueryOperationTouchScope::LinkRelation(
            WorthQueryOperationRelationTouchScope::new(
                binding(),
                "Parent".to_owned(),
                "Occurrence".to_owned(),
                "Occurrence".to_owned(),
            ),
        ));

        for group in [
            ApplicationInvariantGroup::RelationIntegrity,
            ApplicationInvariantGroup::SchemaCompliance,
        ] {
            assert!(requires(&touches, &[group]));
        }
        assert!(!requires(
            &touches,
            &[ApplicationInvariantGroup::LineageIntegrity]
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
