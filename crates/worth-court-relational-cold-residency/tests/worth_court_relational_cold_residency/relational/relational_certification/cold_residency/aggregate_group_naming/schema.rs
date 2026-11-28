//! Source and carrier naming declarations shared by the automatic group court.
use std::collections::BTreeMap;
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, PersistentNameSubjectKind,
};
use worth_relational::facade::schema::*;

pub(super) const FACE: KindId = KindId(1);
pub(super) const VERTEX: KindId = KindId(6);
pub(super) const COUPLING: KindId = KindId(2);
pub(super) const INCIDENCE: KindId = KindId(9);
pub(super) const FACE_NAME: &str = "fixture.face.v1";
pub(super) const INCIDENCE_NAME: &str = "fixture.incidence.v1";
pub(super) const CASES: [&str; 2] = ["Negative", "Positive"];

fn schema() -> SchemaId {
    SchemaId("automatic-groups".into())
}

pub(super) fn domain() -> ProjectedMemberDomainIdentity {
    ProjectedMemberDomainIdentity {
        schema: schema(),
        version: SchemaVersionId(1),
        name: "FaceSide".into(),
    }
}

fn adjacency() -> DerivedRelationIdentity {
    DerivedRelationIdentity {
        schema: schema(),
        version: SchemaVersionId(1),
        name: "SideAdjacency".into(),
    }
}

pub(super) fn descriptor(projected: bool) -> AggregateIdentityContractDeclaration {
    let source = if projected {
        AggregateIdentitySource::DerivedComponents {
            derived_components: adjacency(),
        }
    } else {
        AggregateIdentitySource::Orbit(COUPLING)
    };
    AggregateIdentityContractDeclaration {
        definition: AggregateDefinitionIdentity::from_bytes([if projected { 42 } else { 7 }; 32]),
        source,
        member_kind: FACE,
        membership_kind: KindId(22),
        storage: AggregateIdentityStorage {
            aggregate_kind: KindId(20),
            lineage_kind: KindId(21),
            aggregate_aspect: AspectIdentity(3101),
            lineage_aspect: AspectIdentity(3102),
            projected: projected.then_some(AggregateProjectedMemberStorage {
                coordinate_aspect: AspectIdentity(3103),
                membership_aspect: AspectIdentity(3104),
            }),
        },
    }
}

fn kinds(projected: bool) -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new();
    for (kind_id, kind_name) in [(FACE, "Face"), (VERTEX, "Vertex")] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id,
                kind_name: kind_name.into(),
                schema_id: schema(),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
    }
    let target = if projected { VERTEX } else { FACE };
    let cardinality = if projected {
        Vec::new()
    } else {
        vec![CardinalityContractDeclaration {
            contract_id: "successor-orbit".into(),
            source_min: Some(1),
            source_max: Some(1),
            target_min: Some(1),
            target_max: Some(1),
            pair_min: None,
            pair_max: None,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
        }]
    };
    registry = relation_kind(registry, COUPLING, "Coupling", target, cardinality);
    relation_kind(registry, INCIDENCE, "Incidence", VERTEX, Vec::new())
}

fn relation_kind(
    registry: RelationalSchemaRegistry,
    kind_id: KindId,
    name: &str,
    target: KindId,
    cardinality: Vec<CardinalityContractDeclaration>,
) -> RelationalSchemaRegistry {
    registry
        .register_relation_kind(RelationKindRegistration {
            kind_id,
            kind_name: name.into(),
            schema_id: schema(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: format!("{name}-endpoints").into(),
                    allowed_source_kinds: vec![FACE],
                    allowed_target_kinds: vec![target],
                    self_edges_allowed: target == FACE,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                cardinality,
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap()
}

fn projected_source(registry: RelationalSchemaRegistry) -> RelationalSchemaRegistry {
    use DerivedRelationNode::*;
    let registry = registry
        .register_projected_member_domains(vec![ProjectedMemberDomainParts {
            identity: domain(),
            carrier: FACE,
            cases: CASES.iter().map(|case| (*case).into()).collect(),
        }])
        .unwrap();
    let side = NativeMemberDomain::Projected {
        projected: domain(),
    };
    registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: adjacency(),
            from: side.clone(),
            to: side,
            nodes: vec![
                CarrierLift {
                    domain: domain(),
                    case: CASES[0].into(),
                },
                Reverse { input: 0 },
                Stored {
                    relation: COUPLING,
                    from: FACE,
                    to: VERTEX,
                },
                Reverse { input: 2 },
                Compose { left: 1, right: 2 },
                Compose { left: 4, right: 3 },
                CarrierLift {
                    domain: domain(),
                    case: CASES[1].into(),
                },
                Compose { left: 5, right: 6 },
                Reverse { input: 7 },
                Union { left: 7, right: 8 },
            ],
            laws: Some(DerivedRelationGraphLaws {
                self_edges_allowed: false,
                cross_context_policy: CrossContextPolicy::Forbid,
                cardinality: DerivedRelationCardinality::default(),
                acyclic: false,
                symmetric: true,
                orbit: false,
                same_partition_endpoints: true,
                reaches_at_least: None,
            }),
        }])
        .unwrap()
}

pub(super) fn registry(projected: bool, named: bool) -> RelationalSchemaRegistry {
    let mut registry = kinds(projected);
    if projected {
        registry = projected_source(registry);
    }
    let descriptor = descriptor(projected);
    let version = SchemaVersionId(1);
    let registry =
        register_aggregate_identity_storage(registry, &schema(), version, descriptor.storage)
            .unwrap();
    let registry = register_aggregate_membership(registry, &schema(), version, descriptor).unwrap();
    if !named {
        return registry;
    }
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    let mut bindings = BTreeMap::new();
    for (id, subject, name, kind) in [
        (FACE_NAME, PersistentNameSubjectKind::Entity, "Face", FACE),
        (
            INCIDENCE_NAME,
            PersistentNameSubjectKind::Relation,
            "Incidence",
            INCIDENCE,
        ),
    ] {
        schemes
            .register(
                InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                    id,
                    subject,
                    name,
                    ["authored"],
                    false,
                    true,
                    false,
                )
                .unwrap(),
            )
            .unwrap();
        bindings.insert((subject, kind), id.into());
    }
    registry
        .install_persistent_name_schemes(schemes, bindings)
        .unwrap()
}
