//! Predicate refusal court: unit-bearing comparison predicates and exact
//! duplicate filter clauses refuse cleanly through definition validation
//! without panicking, while distinct upper/lower bounds remain legal.
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_query::{
    ApplicationQueryComparisonOp, ApplicationQueryDefinitionDenial,
    ApplicationQueryWhereFilterClauses, ApplicationQueryWhereFilterDenial,
};
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, worth_query_value_binding, WorthQuery, WorthQueryRecord,
    WorthQueryRow, WorthQuerySchema,
};

use super::queries::ReadyTasksQuery;
use super::schema::{Task, TaskFacts};

#[allow(dead_code)]
mod predicate_refusal_fixtures {
    use super::*;

    const UNIT_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.unit-schema.v1");
    const DUP_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.dup-filter.v1");
    const RANGE_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.range-filter.v1");

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Meter(pub i64);
    impl Meter {
        fn encode(v: &Self) -> i64 {
            v.0
        }
        fn decode(v: i64) -> Option<Self> {
            Some(Self(v))
        }
    }

    worth_query_value_binding! {
        pub MeterBinding for Meter {
            identity: "daymark.sol1.meter.v1",
            scalar: Int64,
            unit: "metre",
            encode: Meter::encode,
            decode: Meter::decode,
        }
    }

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        schema,
        owner = "worth.sol1.unit",
        name = "unit_test",
        members(UnitFragment, UnitPrincipal)
    )]
    pub struct UnitSchema;

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        fragment,
        name = "unit_frag",
        bindings = UnitCodecs,
        members(UnitMapping, UnitPerson, Beam)
    )]
    pub struct UnitFragment;

    worth_query_native_scalar_bindings!(pub UnitCodecs);
    impl Scalar<WorthQueryExternalPrincipalIdentity> for UnitCodecs {
        type Binding = WorthQueryExternalPrincipalIdentityBinding;
        const WRITABLE: bool = false;
    }
    impl Scalar<WorthQueryPrincipalMappingStatus> for UnitCodecs {
        type Binding = WorthQueryPrincipalMappingStatusBinding;
    }
    impl Scalar<Meter> for UnitCodecs {
        type Binding = MeterBinding;
    }

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(UnitMappingFacts), relations(UnitMappingTarget))]
    pub struct UnitMapping;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(UnitPersonFacts))]
    pub struct UnitPerson;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(BeamFacts), reference(UnitSchema, UnitFragment))]
    pub struct Beam;

    #[derive(WorthQuerySchema)]
    #[worth_query(
        relation,
        to = UnitPerson,
        from = UnitMapping,
        reference(UnitSchema, UnitFragment)
    )]
    pub struct UnitMappingTarget;

    #[derive(WorthQueryRecord)]
    #[worth_query(
        bindings = UnitCodecs,
        aspect = UnitMappingFacts,
        fragment = UnitFragment,
        entity = UnitMapping
    )]
    pub struct UnitMappingValue {
        #[worth_query(column = "ExtId", equality)]
        pub identity: WorthQueryExternalPrincipalIdentity,
        #[worth_query(column = "MapStatus", equality)]
        pub status: WorthQueryPrincipalMappingStatus,
    }

    #[derive(WorthQueryRecord)]
    #[worth_query(
        bindings = UnitCodecs,
        aspect = UnitPersonFacts,
        fragment = UnitFragment,
        entity = UnitPerson
    )]
    pub struct UnitPersonValue {
        #[worth_query(column = "PersonId", equality, read_only)]
        pub person_id: u64,
    }

    #[derive(WorthQueryRecord)]
    #[worth_query(
        bindings = UnitCodecs,
        aspect = BeamFacts,
        fragment = UnitFragment,
        entity = Beam
    )]
    pub struct BeamFactsValue {
        #[worth_query(column = "BeamKey", equality, read_only)]
        pub beam_key: u64,
        #[worth_query(equality)]
        pub length: Meter,
        #[worth_query(equality)]
        pub priority: u64,
    }

    #[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
    #[worth_query(schema = UnitSchema, root = Beam, namespace = UNIT_NS)]
    pub struct BeamRow {
        #[worth_query(field = BeamKey)]
        pub beam_key: u64,
    }

    pub struct UnitPrincipal;
    impl UnitPrincipal {
        pub fn reference() -> ApplicationPrincipalBindingRef<
            UnitSchema,
            Self,
            UnitMapping,
            UnitPerson,
            u64,
            U64ApplicationValueBinding,
        > {
            ApplicationPrincipalBindingRef::from_requirements(
                "UnitPrincipal",
                ApplicationPrincipalBindingRequirements {
                    mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                        ExtId::at("ExtId"),
                    ),
                    mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                        MapStatus::at("MapStatus"),
                    ),
                    target: ApplicationPrincipalTargetRequirement::from_relation(
                        UnitMappingTarget::reference(),
                    ),
                    principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                        PersonId::at("PersonId"),
                    ),
                },
            )
        }
    }
    impl Registration<UnitSchema> for UnitPrincipal {
        fn register(
            builder: ApplicationSchemaDeclarationBuilder<UnitSchema>,
        ) -> Result<
            ApplicationSchemaDeclarationBuilder<UnitSchema>,
            ApplicationSchemaDeclarationDenial,
        > {
            Ok(builder.principal_binding(Self::reference()))
        }
    }

    worth_query_structured_value_binding!(pub UnitDenialBinding for () {
        identity: worth_query_identity!(UNIT_NS, denial)
    });

    pub struct UnitContext;
    impl ApplicationBindingContext<UnitSchema> for UnitContext {
        type PrincipalBinding = UnitPrincipal;
        type Mapping = UnitMapping;
        type Principal = UnitPerson;
        type PrincipalIdentity = u64;
        type PrincipalIdentityBinding = U64ApplicationValueBinding;
        type IdempotencyKey = u64;
        type DenialBinding = UnitDenialBinding;
        const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(UNIT_NS, idempotency);
        fn principal_binding() -> ApplicationPrincipalBindingRef<
            UnitSchema,
            UnitPrincipal,
            UnitMapping,
            UnitPerson,
            u64,
            U64ApplicationValueBinding,
        > {
            UnitPrincipal::reference()
        }
    }

    #[derive(Clone, Debug, WorthQuery)]
    #[worth_query(
        schema = UnitSchema,
        context = UnitContext,
        namespace = UNIT_NS,
        result = BeamRow,
        root = Beam,
        scope = BeamKey,
        fields = BeamFacts,
        where_gt(length = Meter(5)),
        limits = 16,
    )]
    pub struct UnitLengthCompare {
        #[worth_query(scope)]
        pub beam: u64,
    }

    #[derive(Clone, Debug, WorthQuery)]
    #[worth_query(
        schema = UnitSchema,
        context = UnitContext,
        namespace = DUP_NS,
        result = BeamRow,
        root = Beam,
        scope = BeamKey,
        fields = BeamFacts,
        where_gt(priority = 1),
        where_gt(priority = 1),
        limits = 16,
    )]
    pub struct DuplicateFilterClause {
        #[worth_query(scope)]
        pub beam: u64,
    }

    #[derive(Clone, Debug, WorthQuery)]
    #[worth_query(
        schema = UnitSchema,
        context = UnitContext,
        namespace = RANGE_NS,
        result = BeamRow,
        root = Beam,
        scope = BeamKey,
        fields = BeamFacts,
        where_gt(priority = 0),
        where_lt(priority = 3),
        limits = 16,
    )]
    pub struct LegalPriorityRange {
        #[worth_query(scope)]
        pub beam: u64,
    }
}

#[test]
fn unit_bearing_and_duplicate_filter_refuse_without_panic_and_range_admits() {
    let unit_err = declaration_builder(predicate_refusal_fixtures::UnitSchema)
        .expect("base UnitSchema builder initializes")
        .application_query(predicate_refusal_fixtures::UnitLengthCompareQuery::definition())
        .build()
        .expect_err("unit-bearing comparison operand must refuse without panicking");
    assert!(
        matches!(
            unit_err,
            ApplicationSchemaDeclarationDenial::InvalidQueryDefinition {
                cause: ApplicationQueryDefinitionDenial::InvalidPredicateValue,
                ..
            }
        ),
        "expected InvalidPredicateValue without panic, got {unit_err:?}"
    );

    let encoded_one = ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
        .expect("u64 encodes");
    let encoded_one_again = ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
        .expect("u64 encodes");
    let direct_dup = ApplicationQueryWhereFilterClauses::<ReadyTasksQuery, Task>::new()
        .compare_literal(
            ApplicationQueryComparisonOp::GreaterThan,
            TaskFacts::priority(),
            encoded_one,
        )
        .ok()
        .expect("first clause admits")
        .compare_literal(
            ApplicationQueryComparisonOp::GreaterThan,
            TaskFacts::priority(),
            encoded_one_again,
        )
        .err()
        .expect("exact duplicate filter clause must refuse before slot allocation");
    assert_eq!(
        direct_dup,
        ApplicationQueryWhereFilterDenial::DuplicateClause
    );

    let macro_dup = declaration_builder(predicate_refusal_fixtures::UnitSchema)
        .expect("base UnitSchema builder initializes")
        .application_query(predicate_refusal_fixtures::DuplicateFilterClauseQuery::definition())
        .build()
        .expect_err("duplicate macro comparison clause must refuse cleanly without panicking");
    assert!(
        matches!(
            macro_dup,
            ApplicationSchemaDeclarationDenial::InvalidQueryDefinition {
                cause: ApplicationQueryDefinitionDenial::WhereFilterUnsupportedShape,
                ..
            }
        ),
        "expected WhereFilterUnsupportedShape without panic, got {macro_dup:?}"
    );

    let legal_range = declaration_builder(predicate_refusal_fixtures::UnitSchema)
        .expect("base UnitSchema builder initializes")
        .application_query(predicate_refusal_fixtures::LegalPriorityRangeQuery::definition())
        .build();
    assert!(
        legal_range.is_ok(),
        "distinct lower and upper bounds on the same field remain legal: {:?}",
        legal_range.err()
    );
}
