//! Selector dependencies lowered from the installed scope, decision-read and elevation contracts.
use super::WorthQueryPrimaryGraphLayout;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::indexes::DerivedIndexId;

type EntityFields = BTreeMap<String, BTreeMap<String, BTreeSet<String>>>;
#[derive(Debug)]
pub(super) struct WorthQueryAuthorizationSelectors {
    fields: BTreeMap<String, EntityFields>,
}
impl WorthQueryAuthorizationSelectors {
    pub(super) fn lower(schema: &ErasedApplicationSchemaDeclaration) -> Self {
        let mut fields = BTreeMap::<String, EntityFields>::new();
        for member in schema.members() {
            if let ApplicationSchemaMember::ApplicationCapability { contract } = member {
                fields.entry(contract.operation().to_owned()).or_default();
            }
        }
        for member in schema.members() {
            match member {
                ApplicationSchemaMember::OperationDecisionRead {
                    operation,
                    target:
                        ApplicationOperationDecisionReadTarget::Field {
                            entity,
                            aspect,
                            field,
                        },
                } => {
                    if let Some(selected) = fields.get_mut(operation) {
                        insert(selected, entity, aspect, field);
                    }
                }
                ApplicationSchemaMember::OperationDecisionRead {
                    operation,
                    target: ApplicationOperationDecisionReadTarget::Entity { entity },
                } => {
                    if let Some(selected) = fields.get_mut(operation) {
                        for member in schema.members() {
                            if let ApplicationSchemaMember::Field {
                                entity: field_entity,
                                aspect,
                                field,
                                equality_queryable: true,
                                ..
                            } = member
                            {
                                if field_entity == entity {
                                    insert(selected, entity, aspect, field);
                                }
                            }
                        }
                    }
                }
                ApplicationSchemaMember::ApplicationMutation { description } => {
                    let parts = description.parts();
                    if let Some(selected) = fields.get_mut(&parts.operation) {
                        let scope = &parts.scope;
                        insert(selected, &scope.entity, &scope.aspect, &scope.field);
                    }
                }
                ApplicationSchemaMember::ApplicationCapability { contract } => {
                    if let Some(revocation) = contract.delegation().revocation() {
                        if let Some(selected) = fields.get_mut(revocation.operation().operation()) {
                            // Installation derives these native reads; applications
                            // are forbidden to author OperationDecisionRead here.
                            for target in worth_query_declaration::facade::application_capability::application_capability_revocation_decision_reads(contract).into_iter().flatten() {
                                if let ApplicationOperationDecisionReadTarget::Field {entity,aspect,field}=target {
                                    insert(selected,&entity,&aspect,&field);
                                }
                            }
                            // The installed target relation binds the native resource
                            // entity. Request selectors may use its declared equality
                            // fields; no application-authored revocation read is needed.
                            for member in schema.members() {
                                if let ApplicationSchemaMember::Field {
                                    entity,
                                    aspect,
                                    field,
                                    equality_queryable: true,
                                    ..
                                } = member
                                {
                                    if entity == contract.target().resource().to() {
                                        insert(selected, entity, aspect, field);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(elevation) = contract.elevation().definition() {
                        // Lifecycle resource selection is structural contract meaning,
                        // not an additional read in the exact native effect graph.
                        if let Some(resource) = elevation.resource_relation() {
                            for transition in elevation.lifecycle().transitions() {
                                if let Some(selected) =
                                    fields.get_mut(transition.operation().operation())
                                {
                                    for member in schema.members() {
                                        if let ApplicationSchemaMember::Field {
                                            entity,
                                            aspect,
                                            field,
                                            equality_queryable: true,
                                            ..
                                        } = member
                                        {
                                            if entity == resource.to() {
                                                insert(selected, entity, aspect, field);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let field = elevation.identity();
                        insert(
                            fields.get_mut(contract.operation()).unwrap(),
                            field.entity(),
                            field.aspect(),
                            field.field(),
                        );
                    }
                }
                _ => {}
            }
        }
        Self { fields }
    }
}
fn insert(selected: &mut EntityFields, entity: &str, aspect: &str, field: &str) {
    selected
        .entry(entity.to_owned())
        .or_default()
        .entry(aspect.to_owned())
        .or_default()
        .insert(field.to_owned());
}
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn authorization_selector_is_declared(
        &self,
        operation: &str,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> bool {
        self.authorization_selectors
            .fields
            .get(operation)
            .is_some_and(|fields| {
                fields
                    .get(entity)
                    .and_then(|aspects| aspects.get(aspect))
                    .is_some_and(|fields| fields.contains(field))
            })
    }
    pub(in crate::domain_computation::primary_graph) fn authorization_selector_index_ids(
        &self,
    ) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.authorization_selectors
            .fields
            .values()
            .flat_map(|entities| entities.iter())
            .flat_map(move |(entity, aspects)| {
                aspects.iter().flat_map(move |(aspect, fields)| {
                    fields.iter().filter_map(move |field| {
                        self.equality_selector_index(entity, aspect, field)
                    })
                })
            })
    }
}
