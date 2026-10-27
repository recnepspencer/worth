//! Actual carriers/incidences/anchors, without a writable logical-member kind.
use super::super::fixtures as physical;
pub(super) use crate::derived_relation::*;
pub(super) use crate::facade::transactions::{
    CreatedEntityRef, EntityReference, EntitySpec, RelationSpec,
};
pub(super) use crate::facade::{
    runtime::{PinnedAggregateRead, PinnedAggregateReadScope},
    snapshots::SnapshotHandle,
};
pub(super) use crate::graph_member::*;
pub(super) use crate::identity::data::EntityId;
pub(super) use crate::projected_member_domain::*;
pub(super) use crate::schema::data::*;
pub(super) use crate::tests::support::*;
pub(super) use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};

pub(super) const CASES: [&str; 2] = ["Negative", "Positive"];
pub(super) const INCIDENCE: u32 = 2;
pub(super) const NEG_REGION: u32 = 9;
pub(super) const POS_REGION: u32 = 10;
pub(super) const NEG_EXTERIOR: u32 = 11;
pub(super) const POS_EXTERIOR: u32 = 12;
pub(super) fn domain() -> ProjectedMemberDomainIdentity {
    ProjectedMemberDomainIdentity {
        schema: SchemaId("logical-components".into()),
        version: SchemaVersionId(1),
        name: "Uses".into(),
    }
}
pub(super) fn identity(name: &str) -> DerivedRelationIdentity {
    DerivedRelationIdentity {
        schema: domain().schema,
        version: SchemaVersionId(1),
        name: name.into(),
    }
}
fn use_domain() -> NativeMemberDomain {
    NativeMemberDomain::Projected {
        projected: domain(),
    }
}
fn parts(
    name: &str,
    to: NativeMemberDomain,
    nodes: Vec<DerivedRelationNode>,
) -> DerivedRelationDefinitionParts {
    DerivedRelationDefinitionParts {
        identity: identity(name),
        from: use_domain(),
        to,
        nodes,
        laws: None,
    }
}
fn incidence_recipe(
    name: &str,
    negative: u32,
    positive: u32,
    target: u32,
) -> DerivedRelationDefinitionParts {
    use DerivedRelationNode::*;
    parts(
        name,
        KindId(target).into(),
        vec![
            CarrierLift {
                domain: domain(),
                case: CASES[0].into(),
            },
            Reverse { input: 0 },
            Stored {
                relation: KindId(negative),
                from: KindId(1),
                to: KindId(target),
            },
            Compose { left: 1, right: 2 },
            CarrierLift {
                domain: domain(),
                case: CASES[1].into(),
            },
            Reverse { input: 4 },
            Stored {
                relation: KindId(positive),
                from: KindId(1),
                to: KindId(target),
            },
            Compose { left: 5, right: 6 },
            Union { left: 3, right: 7 },
        ],
    )
}
pub(super) fn recipes() -> Vec<DerivedRelationDefinitionParts> {
    use DerivedRelationNode::*;
    let mut adjacency = parts(
        "Adjacency",
        use_domain(),
        vec![
            Derived(identity("Vertices")),
            Reverse { input: 0 },
            Compose { left: 0, right: 1 },
            Derived(identity("Regions")),
            Reverse { input: 3 },
            Compose { left: 3, right: 4 },
            Derived(identity("Exteriors")),
            Reverse { input: 6 },
            Compose { left: 6, right: 7 },
            Union { left: 5, right: 8 },
            Intersection { left: 2, right: 9 },
            WithoutSelfEdges { input: 10 },
        ],
    );
    adjacency.laws = Some(DerivedRelationGraphLaws {
        self_edges_allowed: false,
        cross_context_policy: CrossContextPolicy::Forbid,
        cardinality: DerivedRelationCardinality::default(),
        acyclic: false,
        symmetric: true,
        orbit: false,
        same_partition_endpoints: true,
        reaches_at_least: None,
    });
    vec![
        incidence_recipe("Vertices", INCIDENCE, INCIDENCE, 6),
        incidence_recipe("Regions", NEG_REGION, POS_REGION, 7),
        incidence_recipe("Exteriors", NEG_EXTERIOR, POS_EXTERIOR, 8),
        adjacency,
    ]
}
pub(super) fn descriptor() -> AggregateIdentityContractDeclaration {
    AggregateIdentityContractDeclaration {
        definition: AggregateDefinitionIdentity::from_bytes([42; 32]),
        source: AggregateIdentitySource::DerivedComponents {
            derived_components: identity("Adjacency"),
        },
        member_kind: KindId(1),
        membership_kind: KindId(22),
        storage: AggregateIdentityStorage {
            aggregate_kind: KindId(20),
            lineage_kind: KindId(21),
            aggregate_aspect: AspectIdentity(3101),
            lineage_aspect: AspectIdentity(3102),
            projected: Some(AggregateProjectedMemberStorage {
                coordinate_aspect: AspectIdentity(3103),
                membership_aspect: AspectIdentity(3104),
            }),
        },
    }
}
pub(super) fn source_registry() -> RelationalSchemaRegistry {
    source_registry_with_cases(&CASES)
}
pub(super) fn source_registry_with_cases(cases: &[&str]) -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new();
    for (kind, name) in [(1, "carrier"), (6, "vertex"), (7, "region"), (8, "body")] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.into(),
                schema_id: domain().schema,
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
    }
    for (kind, target) in [
        (INCIDENCE, 6),
        (NEG_REGION, 7),
        (POS_REGION, 7),
        (NEG_EXTERIOR, 8),
        (POS_EXTERIOR, 8),
    ] {
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(kind),
                kind_name: format!("incidence-{kind}"),
                schema_id: domain().schema,
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::Forbid,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: format!("endpoints-{kind}").into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(target)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::Forbid,
                    }],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                ),
            })
            .unwrap();
    }
    registry = registry
        .register_projected_member_domains(vec![ProjectedMemberDomainParts {
            identity: domain(),
            carrier: KindId(1),
            cases: cases.iter().map(|case| (*case).to_owned()).collect(),
        }])
        .unwrap();
    registry.register_derived_relations(recipes()).unwrap()
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let registry = crate::schema::aggregate_identity::register_aggregate_identity_storage(
        source_registry(),
        &domain().schema,
        SchemaVersionId(1),
        descriptor().storage,
    )
    .unwrap();
    crate::schema::aggregate_identity::register_aggregate_membership(
        registry,
        &domain().schema,
        SchemaVersionId(1),
        descriptor(),
    )
    .unwrap()
}
pub(super) fn runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build()
}
pub(super) fn created(kind: u32, index: usize) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(format!("entity-{kind}-{index}")),
    }
}
pub(super) fn entity(kind: u32, index: usize) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: created(kind, index).client_key,
        fields: Default::default(),
    }))
}
pub(super) fn edge(
    key: &str,
    kind: u32,
    source: EntityReference,
    target: EntityReference,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(key),
        source,
        target,
        fields: Default::default(),
    }))
}
pub(super) fn commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> CommitResult {
    physical::commit(execution, runtime, "main", batch)
}
pub(super) fn key(carrier: EntityId, side: usize) -> NativeMemberKey {
    NativeMemberKey::Projected(std::sync::Arc::new(ProjectedMemberCoordinate {
        domain: domain(),
        carrier,
        case: CASES[side].into(),
    }))
}
pub(super) fn read(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    member: NativeMemberKey,
) -> PinnedAggregateRead<NativeMemberKey> {
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .unwrap()
        .read_projected_aggregate(
            execution,
            PinnedAggregateReadScope::for_member_definition(descriptor().definition, member),
        )
        .unwrap()
}
