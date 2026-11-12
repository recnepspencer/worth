//! History and invariant Source declarations refer to actual query/index members.
use super::*;
use std::num::NonZeroU64;
use worth_query_declaration::facade::{application_history::*, application_query::*};

pub(super) fn extend(members: &mut Vec<ApplicationSchemaMember>) {
    let access = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ApplicationResource { definition } => {
                Some(definition.parts().access.clone())
            }
            _ => None,
        })
        .unwrap();
    let root = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ApplicationQuery { definition }
                if definition.name() == access.parts().query_name =>
            {
                Some(definition.root_entity().to_owned())
            }
            _ => None,
        })
        .unwrap();
    let aspect = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::Aspect {
                entity,
                aspect,
                identity,
                revision,
                ..
            } if entity == &root => Some(ApplicationHistoryAspectSelection {
                aspect: aspect.clone(),
                identity: *identity,
                revision: *revision,
            }),
            _ => None,
        })
        .unwrap();
    let index = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ManagedIndex { definition } => Some(definition.clone()),
            _ => None,
        })
        .unwrap();
    let (binding, mutation_scope) = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ApplicationMutation { description } => Some((
                description.binding_identity().as_str().to_owned(),
                description.scope().entity.clone(),
            )),
            _ => None,
        })
        .unwrap();
    let ownership = ApplicationHistoryOwnershipRelation {
        relation: "BudgetHistoryMembers".into(),
        from: access.parts().scope_entity.clone(),
        to: mutation_scope,
    };
    members.push(ApplicationSchemaMember::Relation {
        relation: ownership.relation.clone(),
        from: ownership.from.clone(),
        to: ownership.to.clone(),
        integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into(),
    });
    let mut audience = members
        .iter()
        .filter_map(|m| match m {
            ApplicationSchemaMember::Entity { entity } => {
                Some(ApplicationInvariantScopeTarget::Entity(entity.clone()))
            }
            ApplicationSchemaMember::Relation { relation, .. } => {
                Some(ApplicationInvariantScopeTarget::Relation(relation.clone()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    audience.sort();
    audience.dedup();
    members.extend([
        ApplicationSchemaMember::ApplicationHistory {
            definition: ErasedApplicationHistoryDefinition::from_untrusted_parts(
                WorthQueryPortableApplicationHistoryParts {
                    identity: "BudgetModelHistory".into(),
                    history_type: portable("worth.budget.model-history"),
                    access: access.clone(),
                    metadata_disclosure: ApplicationQueryDisclosureContract::public(),
                    model_domain: ApplicationHistoryModelDomain::OwnedModelGraph,
                    ownership: vec![ownership],
                    navigation: None,
                    window: None,
                    bookmarks: None,
                    operations: vec![ApplicationHistoryOperation {
                        binding,
                        label: "Inspect budget body".into(),
                    }],
                },
            ),
        },
        ApplicationSchemaMember::ApplicationHistoryQuery {
            definition: ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(
                WorthQueryPortableApplicationHistoryQueryParts {
                    name: "BudgetRecordHistory".into(),
                    history_type: portable("worth.budget.record-history"),
                    subject_read: access,
                    target: ApplicationHistoryTarget::Entity { entity: root },
                    aspects: vec![aspect],
                    metadata_disclosure: ApplicationQueryDisclosureContract::public(),
                },
            ),
        },
        ApplicationSchemaMember::ApplicationInvariant {
            invariant: "BudgetInvariant".into(),
            major: 1,
            minor: 0,
            execution_point: ApplicationInvariantExecutionPoint::CommitBoundary,
            maximum_work_units: NonZeroU64::new(128),
            enforcement: ApplicationInvariantEnforcement::BlockCommit,
            required_groups: vec![ApplicationInvariantGroup::SchemaCompliance],
            read_closure: audience.clone(),
            applicability: audience,
            provider: "budget-fixture".into(),
            cost_posture: ApplicationInvariantCostPosture::Touched,
        },
        ApplicationSchemaMember::InvariantManagedIndexSource {
            invariant: "BudgetInvariant".into(),
            execution_point: ApplicationInvariantExecutionPoint::CommitBoundary,
            requirement: ApplicationInvariantManagedIndexSource::from_untrusted_parts(
                index,
                ApplicationManagedIndexColdSource::Refuse,
            ),
        },
    ]);
}

#[derive(worth_foundational::facade::WorthValue)]
pub struct BudgetProbe {
    prefix: String,
}
worth_query_declaration::worth_query_value_binding!(pub BudgetProbeBinding for BudgetProbe {
    identity: "worth.budget.index-probe", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});
struct BudgetIndex;
impl ApplicationManagedIndexBinding<crate::application_principal_binding_tests::IdentitySchema>
    for BudgetIndex
{
    type SourceBinding = crate::application_principal_binding_tests::PrincipalQueryBinding;
    type ProbeBinding = BudgetProbeBinding;
    type Entity = crate::application_principal_binding_tests::Principal;
    const NAME: &'static str = "BudgetPrincipalIndex";
    const PROVIDER: &'static str = "worth.budget.principal-provider";
    const PROVIDER_VERSION: u32 = 1;
}

pub(super) fn index_declaration() -> ErasedApplicationSchemaDeclaration {
    crate::application_principal_binding_tests::schema_members::declare()
        .managed_index(ApplicationManagedIndexDefinition::<
            crate::application_principal_binding_tests::IdentitySchema,
            BudgetIndex,
        >::new())
        .build()
        .unwrap()
        .erased()
        .clone()
}
