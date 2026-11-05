//! Authored, nonempty Source and native-rule metadata; no Source receiver.
use std::sync::{atomic::AtomicUsize, Arc};
use worth_relational::facade::{identity::KindId, runtime::*, schema::*};

pub(super) fn inventory(
    source: &RelationalSchemaRegistry,
    routes: usize,
    evaluated: Arc<AtomicUsize>,
) -> (
    RelationalSchemaRegistry,
    Vec<CustomInvariantRegistration>,
    Vec<ManagedIndexSourceProgram>,
) {
    let identity = DerivedRelationIdentity {
        schema: SchemaId("oracle".into()),
        version: SchemaVersionId(1),
        name: "compilation-membership".into(),
    };
    let registry = native_registry(source)
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity.clone(),
            from: KindId(1).into(),
            to: KindId(1).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(1),
            }],
            laws: None,
        }])
        .unwrap();
    let path = registry.derived_relation(&identity).unwrap();
    let routes = (0..routes)
        .map(|index| {
            ManagedIndexSourceProjectionRoute::new(
                format!("endpoint-{index}"),
                path.clone(),
                Vec::new(),
            )
            .unwrap()
        })
        .collect();
    let program = registry
        .bind_managed_index_source(&path, Vec::new(), routes)
        .unwrap();
    let invariant = CustomInvariantRegistration::new(SourceMetadataRule(
        super::super::transition_executor::TargetRule(evaluated),
    ))
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
    .unwrap();
    (registry, vec![invariant], vec![program])
}
struct SourceMetadataRule(super::super::transition_executor::TargetRule);
impl CustomInvariantRule for SourceMetadataRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = self.0.descriptor();
        descriptor.operational.access = CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(1)],
            read_relation_kinds: vec![KindId(2)],
            affected_entity_kinds: vec![KindId(1)],
            affected_relation_kinds: vec![KindId(2)],
            include_relation_endpoint_entity_touches: true,
        };
        descriptor
    }
    fn prepare_scope(
        &self,
        scope: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        self.0.prepare_scope(scope)
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.0.evaluate(context, scope)
    }
}

pub(super) fn native_registry(source: &RelationalSchemaRegistry) -> RelationalSchemaRegistry {
    let mut relation = source.relation_registration(KindId(2)).unwrap().clone();
    relation.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "compiled-source-endpoints".into(),
            allowed_source_kinds: vec![KindId(1)],
            allowed_target_kinds: vec![KindId(1)],
            self_edges_allowed: true,
            cross_context_policy: relation.cross_context_policy,
        }],
        vec![CardinalityContractDeclaration {
            contract_id: "compiled-source-cardinality".into(),
            source_max: Some(2),
            target_max: None,
            pair_max: None,
            source_min: Some(1),
            target_min: None,
            pair_min: None,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    RelationalSchemaRegistry::new()
        .register_entity_kind(source.entity_registration(KindId(1)).unwrap().clone())
        .unwrap()
        .register_relation_kind(relation)
        .unwrap()
}
