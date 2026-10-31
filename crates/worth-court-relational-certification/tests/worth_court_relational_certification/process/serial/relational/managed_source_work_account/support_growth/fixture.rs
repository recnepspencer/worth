//! Authored composed Source and direct projection path over genuine Native IDs.
use super::*;
use worth_relational::facade::{config::*, schema::*};

pub(super) fn registry() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let (registry, _) = super::super::registry();
    let schema = SchemaId("managed-source-court".into());
    let registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(3),
            kind_name: "item-self".into(),
            schema_id: schema.clone(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "self-endpoints".into(),
                    allowed_source_kinds: vec![KindId(4)],
                    allowed_target_kinds: vec![KindId(4)],
                    self_edges_allowed: true,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let identity = |name: &str| DerivedRelationIdentity {
        schema: schema.clone(),
        version: SchemaVersionId(1),
        name: name.into(),
    };
    let edge = DerivedRelationNode::Stored {
        relation: KindId(3),
        from: KindId(4),
        to: KindId(4),
    };
    let composed = identity("composed-items");
    let projection = identity("self-projection");
    let registry = registry
        .register_derived_relations(vec![
            DerivedRelationDefinitionParts {
                identity: composed.clone(),
                from: KindId(1).into(),
                to: KindId(4).into(),
                nodes: vec![
                    DerivedRelationNode::Stored {
                        relation: KindId(2),
                        from: KindId(1),
                        to: KindId(4),
                    },
                    edge.clone(),
                    DerivedRelationNode::Compose { left: 0, right: 1 },
                ],
                laws: None,
            },
            DerivedRelationDefinitionParts {
                identity: projection.clone(),
                from: KindId(4).into(),
                to: KindId(4).into(),
                nodes: vec![edge],
                laws: None,
            },
        ])
        .unwrap();
    let program = registry
        .bind_managed_index_source(
            &registry.derived_relation(&composed).unwrap(),
            vec![],
            vec![ManagedIndexSourceProjectionRoute::new(
                "self".into(),
                registry.derived_relation(&projection).unwrap(),
                vec![],
            )
            .unwrap()],
        )
        .unwrap();
    (registry, program)
}

pub(super) fn batch(count: usize) -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("support-fanout").push(entity(1, "scope"));
    // The extra self-linked item has no contains edge: its inverse Source is
    // empty, but its projection and negative inverse support remain required.
    for index in 0..=count {
        let item = format!("item-{index}");
        batch = batch.push(entity(4, &item));
        if index < count {
            batch = batch.push(relation(
                2,
                &format!("contains-{index}"),
                created(1, "scope"),
                created(4, &item),
            ));
        }
        batch = batch.push(relation(
            3,
            &format!("self-{index}"),
            created(4, &item),
            created(4, &item),
        ));
    }
    batch
}
fn entity(kind: u32, label: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        fields: Default::default(),
    }))
}
fn relation(
    kind: u32,
    label: &str,
    source: CreatedEntityRef,
    target: CreatedEntityRef,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        source: EntityReference::Created(source),
        target: EntityReference::Created(target),
        fields: Default::default(),
    }))
}

pub(super) struct Reader(pub owner::Reader);
impl CustomInvariantRule for Reader {
    type Scope = Arc<PreparedManagedIndexSourceRead>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = self.0.descriptor();
        descriptor
            .operational
            .access
            .read_relation_kinds
            .push(KindId(3));
        descriptor
            .operational
            .access
            .affected_relation_kinds
            .push(KindId(3));
        descriptor
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.0.prepare_scope(planner)
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.0.evaluate(context, scope)
    }
}
