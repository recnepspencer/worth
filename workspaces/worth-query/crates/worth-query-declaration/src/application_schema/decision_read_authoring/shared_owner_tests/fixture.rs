//! Minimal schema fixture with three managed-index Parents over one governing relation.
use super::super::super::fragment::{self, *};
use super::super::super::*;
use crate::application_query::*;
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub(super) struct TestSchema;
impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "SharedOwnerDedup";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        base().build()
    }
}

pub(super) struct UnitBinding;
impl ApplicationStructuredValueBinding for UnitBinding {
    type Value = ();
    const IDENTITY_NAME: &'static str = "worth.rust.unit";
    fn validate(_: &()) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct Op;
impl ApplicationOperationMarkerIdentity<TestSchema> for Op {
    type InputBinding = UnitBinding;
    const IDENTIFIER: &'static str = "Read";
}

pub(super) struct TestModel;
impl Fragment for TestModel {
    const NAME: &'static str = "test_model";
    type Bindings = TestBindings;
    type Members = (Owner, Member);
}
pub(super) struct TestBindings;
impl BindingSet for TestBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for TestBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for TestBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for TestBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub(super) struct Owner;
impl Entity for Owner {
    const NAME: &'static str = "Owner";
    type Aspects = (OwnerAuth,);
    type Relations = (Gov,);
}
pub(super) struct Member;
impl Entity for Member {
    const NAME: &'static str = "Member";
    type Aspects = (MemberIdentity,);
    type Relations = ();
}
pub(super) struct OwnerAuth;
impl Aspect for OwnerAuth {
    const NAME: &'static str = "Auth";
    type Row = OwnerAuthRow;
}
pub(super) struct OwnerAuthRow {
    id: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Record<OwnerAuthRow> for TestBindings {
    fn record() -> impl Shape<OwnerAuthRow, Self> {
        fragment::field("external_id", |r: &OwnerAuthRow| r.id.clone())
            .equality_queryable(true)
            .and("status", |r: &OwnerAuthRow| r.status)
            .equality_queryable(true)
            .build(|id, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OwnerAuthRow { id, status })
            })
    }
}
pub(super) struct MemberIdentity;
impl Aspect for MemberIdentity {
    const NAME: &'static str = "Identity";
    type Row = u64;
}
pub(super) struct Gov;
impl fragment::Relation for Gov {
    const NAME: &'static str = "Gov";
    type To = Member;
}
impl OperationReads<Op> for Gov {}

type ScopeField = FieldSelection<TestModel, Member, MemberIdentity, u64>;
pub(super) struct PrincipalBindingMarker;

macro_rules! parent_binding {
    ($query:ident, $binding:ident, $qname:literal, $qtype:literal, $bid:literal) => {
        pub(super) struct $query;
        impl ApplicationQueryMarkerIdentity<TestSchema> for $query {
            type ParameterBinding = UnitBinding;
            type ResultBinding = UnitBinding;
            type Scope = Member;
            const IDENTIFIER: &'static str = $qname;
            const QUERY_TYPE_NAME: &'static str = $qtype;
            const SCOPE_TYPE_NAME: &'static str = "Member";
        }
        pub(super) struct $binding;
        impl ApplicationQueryBinding<TestSchema> for $binding {
            type Input = ();
            type InputBinding = UnitBinding;
            type Query = $query;
            type ParameterBinding = UnitBinding;
            type ResultBinding = UnitBinding;
            type ScopeBinding = ApplicationQueryFieldScope<ScopeField>;
            type PrincipalBinding = PrincipalBindingMarker;
            type Mapping = Owner;
            type Principal = Member;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            const IDENTITY: &'static str = $bid;
            const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
            fn scope_field() -> ScopeField {
                ScopeField::at("Identity")
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                TestSchema,
                PrincipalBindingMarker,
                Owner,
                Member,
                u64,
                U64ApplicationValueBinding,
            > {
                ApplicationPrincipalBindingRef::from_requirements(
                    "PrincipalBinding",
                    ApplicationPrincipalBindingRequirements {
                        mapping_identity:
                            ApplicationPrincipalMappingIdentityRequirement::from_field(
                                FieldSelection::<
                                    TestModel,
                                    Owner,
                                    OwnerAuth,
                                    WorthQueryExternalPrincipalIdentity,
                                >::at("external_id"),
                            ),
                        mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                            FieldSelection::<
                                TestModel,
                                Owner,
                                OwnerAuth,
                                WorthQueryPrincipalMappingStatus,
                            >::at("status"),
                        ),
                        target: ApplicationPrincipalTargetRequirement::from_relation(gov_ref()),
                        principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                            ScopeField::at("Identity"),
                        ),
                    },
                )
            }
        }
    };
}

parent_binding!(
    VertexQuery,
    VertexBoxes,
    "vertex_boxes",
    "test.vertex-boxes.query.v1",
    "test.vertex-boxes.binding.v1"
);
parent_binding!(
    EdgeQuery,
    EdgeBoxes,
    "edge_boxes",
    "test.edge-boxes.query.v1",
    "test.edge-boxes.binding.v1"
);
parent_binding!(
    FaceQuery,
    FaceBoxes,
    "face_boxes",
    "test.face-boxes.query.v1",
    "test.face-boxes.binding.v1"
);

#[derive(worth_foundational::facade::WorthValue)]
struct Probe {
    key: String,
}
crate::worth_query_value_binding!(ProbeBinding for Probe {
    identity: "test.shared-owner.probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

fn query_parts(
    name: &str,
    query_type: &'static str,
    probe: Option<&ApplicationManagedIndexProbeMeaning>,
    index: Option<&str>,
) -> WorthQueryPortableApplicationQueryParts {
    let unit = WorthQueryPortableTypeIdentity::declared("worth.rust.unit");
    let qt = WorthQueryPortableTypeIdentity::declared(query_type);
    WorthQueryPortableApplicationQueryParts {
        name: name.into(),
        query_type: qt.clone(),
        parameter_type: unit.clone(),
        result_type: unit.clone(),
        scope_type: WorthQueryPortableTypeIdentity::declared("Member"),
        root_entity: "Member".into(),
        scope_entity: "Member".into(),
        parameters: probe
            .map(|p| {
                vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
                    "probe".into(),
                    p.parts().scalar_family,
                    p.parts().binding.clone(),
                    None,
                    None,
                )]
            })
            .unwrap_or_default(),
        result_shape: ApplicationQueryResultShape::from_untrusted_parts(
            WorthQueryPortableApplicationQueryResultShapeParts {
                query_type: qt,
                root_entity: "Member".into(),
                result_type: unit,
                fields: Vec::new(),
                relations: Vec::new(),
            },
        ),
        root_paths: Vec::new(),
        component_selection: None,
        orbit: None,
        managed_index: index.map(|idx| {
            ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
                WorthQueryPortableQueryManagedIndexParts {
                    semantic_version: 1,
                    index: idx.into(),
                    probe_parameter: "probe".into(),
                },
            )
        }),
        cardinality: ApplicationQueryCardinality::Many,
        predicates: Vec::new(),
        where_filter: None,
        ordering: Vec::new(),
        continuation: None,
        live_cause: None,
        disclosure: ApplicationQueryDisclosureContract::public(),
        authorization: ApplicationQueryAuthorizationRequirement::Public,
        basis_support: ApplicationQueryBasisSupport::current_and_pinned(),
        lanes: ApplicationQueryLaneEligibility::one_shot(),
    }
}

pub(super) fn base() -> ApplicationSchemaDeclarationBuilder<TestSchema> {
    let probe = ApplicationManagedIndexProbeMeaning::of::<ProbeBinding>();
    let source = query_parts("boxes_source", "test.boxes-source.query.v1", None, None);
    let source_binding = ApplicationQueryBindingReference::from_untrusted_parts(
        ApplicationQueryBindingReferenceParts {
            binding: WorthQueryPortableTypeIdentity::declared("test.boxes-source.binding.v1"),
            input: WorthQueryPortableTypeIdentity::declared("worth.rust.unit"),
            query_name: source.name.clone(),
            query: source.query_type.clone(),
            parameters: source.parameter_type.clone(),
            result: source.result_type.clone(),
            scope: source.scope_type.clone(),
            scope_entity: "Member".into(),
            scope_aspect: "Identity".into(),
            scope_field: "Identity".into(),
            scope_value: WorthQueryPortableTypeIdentity::declared("worth.rust.u64"),
            scope_resolution: ApplicationQueryScopeResolutionMode::InputField,
            principal_binding: "PrincipalBinding".into(),
        },
    );
    let idx = |name: &str| ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
            WorthQueryPortableManagedIndexParts {
                semantic_version: 3,
                name: name.into(),
                provider: "test.shared-owner.provider.v1".into(),
                provider_version: 1,
                structure: ApplicationManagedIndexSourceStructure::Opaque,
                membership: ApplicationManagedIndexMembership::Candidates,
                source: ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(
                    source_binding.clone(),
                ),
                entity: "Member".into(),
                probe: probe.clone(),
            },
        ),
    };
    let parent =
        |name: &str, qt: &'static str, index: &str| ApplicationSchemaMember::ApplicationQuery {
            definition: ErasedApplicationQueryDefinition::from_untrusted_parts(query_parts(
                name,
                qt,
                Some(&probe),
                Some(index),
            )),
        };
    ApplicationSchemaDeclarationBuilder::<TestSchema>::from_test_members(vec![
        ApplicationSchemaMember::Entity {
            entity: "Owner".into(),
        },
        ApplicationSchemaMember::Entity {
            entity: "Member".into(),
        },
        ApplicationSchemaMember::Relation {
            relation: "Gov".into(),
            from: "Owner".into(),
            to: "Member".into(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .into(),
        },
        ApplicationSchemaMember::Operation {
            operation: "Read".into(),
            input_type: WorthQueryPortableTypeIdentity::declared("worth.rust.unit"),
        },
        ApplicationSchemaMember::ApplicationQuery {
            definition: ErasedApplicationQueryDefinition::from_untrusted_parts(source),
        },
        idx("VertexIdx"),
        idx("EdgeIdx"),
        idx("FaceIdx"),
        parent("vertex_boxes", "test.vertex-boxes.query.v1", "VertexIdx"),
        parent("edge_boxes", "test.edge-boxes.query.v1", "EdgeIdx"),
        parent("face_boxes", "test.face-boxes.query.v1", "FaceIdx"),
    ])
}

pub(super) fn op_ref() -> ApplicationOperationRef<TestSchema, Op, ()> {
    ApplicationOperationRef::from_declaration()
}

pub(super) fn gov_ref() -> ApplicationRelationRef<TestSchema, Gov, Owner, Member> {
    ApplicationRelationRef::from_schema_identifiers(
        "Gov",
        "Owner",
        "Member",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}

pub(super) fn add_shared<B: ApplicationQueryBinding<TestSchema>>(
    b: ApplicationSchemaDeclarationBuilder<TestSchema>,
) -> ApplicationSchemaDeclarationBuilder<TestSchema> {
    b.operation_shared_owner_managed_index_read::<B, _, _, _, _, _>(op_ref(), gov_ref())
}
