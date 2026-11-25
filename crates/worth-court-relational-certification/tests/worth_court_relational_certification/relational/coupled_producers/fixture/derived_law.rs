//! A world whose schema derives a relation with graph laws from a stored
//! `uses` relation, so each candidate pin projects the aggregate owner's
//! derived laws over the records the proposal changed.
use super::*;
use worth_relational::facade::config::CrossContextPolicy;

/// The stored relation the derived relation reads.
pub const USES: KindId = KindId(9);

/// What a world adds to its schema and runtime beyond its aspects.
pub(super) struct Extra {
    pub(super) rule: Option<CustomInvariantRegistration>,
    derived_law: bool,
}
pub(super) const BARE: Extra = Extra {
    rule: None,
    derived_law: false,
};
impl Extra {
    pub(super) fn rule(rule: CustomInvariantRegistration) -> Self {
        Self {
            rule: Some(rule),
            derived_law: false,
        }
    }
    pub(super) fn extend(
        &self,
        registry: RelationalSchemaRegistry,
        fixture: &AspectSchemaFixture,
    ) -> RelationalSchemaRegistry {
        if !self.derived_law {
            return registry;
        }
        let (schema, version) = (fixture.schema_id.clone(), fixture.schema_version_id);
        let endpoints = EndpointKindContractDeclaration {
            contract_id: "uses:endpoints".into(),
            allowed_source_kinds: vec![KindId(1)],
            allowed_target_kinds: vec![KindId(1)],
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        };
        let registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: USES,
                kind_name: "uses".into(),
                schema_id: schema.clone(),
                schema_version_id: version,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: fixture.cascade_delete_policy,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![endpoints],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                ),
            })
            .unwrap();
        registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: DerivedRelationIdentity {
                    schema,
                    version,
                    name: "used-by".into(),
                },
                from: KindId(1).into(),
                to: KindId(1).into(),
                nodes: vec![DerivedRelationNode::Stored {
                    relation: USES,
                    from: KindId(1),
                    to: KindId(1),
                }],
                laws: Some(DerivedRelationGraphLaws {
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                    cardinality: DerivedRelationCardinality::default(),
                    acyclic: false,
                    symmetric: false,
                    orbit: false,
                    same_partition_endpoints: false,
                    reaches_at_least: None,
                }),
            }])
            .unwrap()
    }
}

impl World {
    /// An owned-source world whose schema derives `used-by` from `USES`
    /// under graph laws.
    pub fn with_derived_law(execution: &ExecutionRequest<'_, '_>) -> Self {
        let extra = Extra {
            rule: None,
            derived_law: true,
        };
        Self::with_aspects(execution, true, true, false, false, false, extra, RETAIN)
    }
}
