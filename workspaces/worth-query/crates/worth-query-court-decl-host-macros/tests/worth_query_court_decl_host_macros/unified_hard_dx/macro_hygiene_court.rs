//! Macro hygiene court: generated macro names must never capture caller
//! constants (`value`, `row`, `related`) or type aliases (`Q`, `B`).
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQueryRelation,
    WorthQueryRow, WorthQuerySchema,
};

#[allow(non_upper_case_globals, dead_code)]
mod hygiene_collision {
    use super::*;

    const value: i32 = 7;
    const row: i32 = 7;
    const related: i32 = 7;
    const __worth_record_field_0: i32 = 7;
    const __worth_record_row: i32 = 7;
    type Q = ();
    type B = ();

    const HYG_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.hygiene.v1");
    const HYG_ROW_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.hygiene-row.v1");
    const HYG_CHILD_NS: ApplicationIdentityNamespace =
        worth_query_identity_namespace!("daymark.sol1.hygiene-child.v1");

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        schema,
        owner = "worth.sol1.hygiene",
        name = "hygiene_schema",
        members(HygFragment, HygPrincipal)
    )]
    pub struct HygSchema;

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        fragment,
        name = "hyg_frag",
        bindings = HygCodecs,
        members(HygMapping, HygPerson, HygProject, HygTask)
    )]
    pub struct HygFragment;

    worth_query_native_scalar_bindings!(pub HygCodecs);
    impl Scalar<WorthQueryExternalPrincipalIdentity> for HygCodecs {
        type Binding = WorthQueryExternalPrincipalIdentityBinding;
        const WRITABLE: bool = false;
    }
    impl Scalar<WorthQueryPrincipalMappingStatus> for HygCodecs {
        type Binding = WorthQueryPrincipalMappingStatusBinding;
    }

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(HygMapFacts), relations(HygMapTarget))]
    pub struct HygMapping;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(HygPersonFacts))]
    pub struct HygPerson;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        entity,
        aspects(HygProjectFacts),
        relations(HygContains),
        reference(HygSchema, HygFragment)
    )]
    pub struct HygProject;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        entity,
        aspects(HygTaskFacts),
        relations(HygBlockedBy),
        reference(HygSchema, HygFragment)
    )]
    pub struct HygTask;

    #[derive(WorthQuerySchema)]
    #[worth_query(
        relation,
        to = HygPerson,
        from = HygMapping,
        reference(HygSchema, HygFragment)
    )]
    pub struct HygMapTarget;

    #[derive(WorthQueryRelation)]
    #[worth_query(to = HygTask, from = HygProject, reference(HygSchema, HygFragment))]
    pub struct HygContains;

    #[derive(WorthQueryRelation)]
    #[worth_query(to = HygTask, from = HygTask, reference(HygSchema, HygFragment))]
    pub struct HygBlockedBy;

    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = HygCodecs, aspect = HygMapFacts, fragment = HygFragment, entity = HygMapping)]
    pub struct HygMapValue {
        #[worth_query(column = "HygExtId", equality)]
        pub identity: WorthQueryExternalPrincipalIdentity,
        #[worth_query(column = "HygMapStatus", equality)]
        pub status: WorthQueryPrincipalMappingStatus,
    }

    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = HygCodecs, aspect = HygPersonFacts, fragment = HygFragment, entity = HygPerson)]
    pub struct HygPersonValue {
        #[worth_query(column = "HygPersonId", equality, read_only)]
        pub person_id: u64,
    }

    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = HygCodecs, aspect = HygProjectFacts, fragment = HygFragment, entity = HygProject)]
    pub struct HygProjectValue {
        #[worth_query(column = "HygProjectKey", equality, read_only)]
        pub project_key: u64,
    }

    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = HygCodecs, aspect = HygTaskFacts, fragment = HygFragment, entity = HygTask)]
    pub struct HygTaskValue {
        #[worth_query(column = "HygTaskKey", equality, read_only)]
        pub task_key: u64,
        #[worth_query(equality)]
        pub state: String,
        #[worth_query(equality)]
        pub priority: u64,
        pub row: String,
        pub value: u64,
    }

    pub struct HygPrincipal;
    impl HygPrincipal {
        pub fn reference() -> ApplicationPrincipalBindingRef<
            HygSchema,
            Self,
            HygMapping,
            HygPerson,
            u64,
            U64ApplicationValueBinding,
        > {
            ApplicationPrincipalBindingRef::from_requirements(
                "HygPrincipal",
                ApplicationPrincipalBindingRequirements {
                    mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                        HygExtId::at("HygExtId"),
                    ),
                    mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                        HygMapStatus::at("HygMapStatus"),
                    ),
                    target: ApplicationPrincipalTargetRequirement::from_relation(
                        HygMapTarget::reference(),
                    ),
                    principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                        HygPersonId::at("HygPersonId"),
                    ),
                },
            )
        }
    }
    impl Registration<HygSchema> for HygPrincipal {
        fn register(
            builder: ApplicationSchemaDeclarationBuilder<HygSchema>,
        ) -> Result<
            ApplicationSchemaDeclarationBuilder<HygSchema>,
            ApplicationSchemaDeclarationDenial,
        > {
            Ok(builder.principal_binding(Self::reference()))
        }
    }

    worth_query_structured_value_binding!(pub HygUnitBinding for () {
        identity: worth_query_identity!(HYG_NS, result)
    });
    worth_query_structured_value_binding!(pub HygDenialBinding for () {
        identity: worth_query_identity!(HYG_NS, denial)
    });

    pub struct HygContext;
    impl ApplicationBindingContext<HygSchema> for HygContext {
        type PrincipalBinding = HygPrincipal;
        type Mapping = HygMapping;
        type Principal = HygPerson;
        type PrincipalIdentity = u64;
        type PrincipalIdentityBinding = U64ApplicationValueBinding;
        type IdempotencyKey = u64;
        type DenialBinding = HygDenialBinding;
        const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(HYG_NS, idempotency);
        fn principal_binding() -> ApplicationPrincipalBindingRef<
            HygSchema,
            HygPrincipal,
            HygMapping,
            HygPerson,
            u64,
            U64ApplicationValueBinding,
        > {
            HygPrincipal::reference()
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
    #[worth_query(schema = HygSchema, root = HygTask, namespace = HYG_CHILD_NS)]
    pub struct HygChildRow {
        #[worth_query(field = HygTaskKey)]
        pub task_key: u64,
    }

    #[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
    #[worth_query(schema = HygSchema, root = HygTask, namespace = HYG_ROW_NS)]
    pub struct HygienicRelatedTaskRow {
        #[worth_query(field = HygTaskKey)]
        pub task_key: u64,
        #[worth_query(relation = HygBlockedBy, nested = HygUnitBinding)]
        pub blockers: Vec<HygChildRow>,
    }

    #[derive(Clone, Debug, WorthQuery)]
    #[worth_query(
        schema = HygSchema,
        context = HygContext,
        namespace = HYG_NS,
        result = HygienicRelatedTaskRow,
        root = HygTask,
        scope = HygProjectKey,
        fields = HygTaskFacts,
        root_path(HygContains),
        where_equal(state = "OPEN"),
        where_gt(priority = 0),
        where_optional(priority = priority),
        where_no_related(HygBlockedBy, state = "OPEN"),
        order_by(HygienicRelatedTaskRow::task_key(), Ascending),
        live = whole_result,
        limits = 32,
    )]
    pub struct HygienicQueryWithLiterals {
        #[worth_query(scope)]
        pub project: u64,
        pub priority: Option<u64>,
    }

    pub fn verify_definition_admits() {
        let _ = (value, row, related, std::mem::size_of::<(Q, B)>());
        let declaration = declaration_builder(HygSchema)
            .expect("HygSchema builder initializes")
            .application_query(HygienicQueryWithLiteralsQuery::definition())
            .build();
        assert!(
            declaration.is_ok(),
            "hygienic derive with colliding caller names must admit: {:?}",
            declaration.err()
        );
    }
}

#[test]
fn macro_hygiene_with_colliding_value_row_related_and_type_aliases() {
    hygiene_collision::verify_definition_admits();
}
