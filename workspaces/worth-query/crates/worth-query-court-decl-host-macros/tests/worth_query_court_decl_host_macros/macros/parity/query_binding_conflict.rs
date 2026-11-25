//! Negative courts for Blocker 2: `bound_application_query` reuses an exact
//! matching binding descriptor (e.g. from a correction member's disclosure),
//! while still refusing duplicate query definitions and conflicting binding
//! meanings sharing an identity.
use super::super::reads::{RowBinding, TaskRow};
use super::super::schema::*;
use super::{board, GenericBoardContext, GenericBoardPrincipal};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{entity, relation, Registration};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_query_binding,
    worth_query_structured_value_binding, WorthQuery, WorthQuerySchema,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::application_installation::{
    in_memory_program, ApplicationBuildIdentity, WorthQueryApplicationLimits,
};
use worth_query_host::facade::worth_query_application_program;

const CONFLICT_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.conflict-query.v1");

#[derive(Clone, Debug, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = CONFLICT_NS,
    query = DuplicateQuery,
    binding = DuplicateBinding,
    result = RowBinding,
    root = Task,
    limits = 2,
    name = "duplicate_query",
)]
pub struct DuplicateReq {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
}

mod conflicting {
    use super::*;

    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        schema,
        owner = "macros.parity.conflict",
        name = "conflict_board",
        members(ConflictContribution)
    )]
    pub struct ConflictSchema;

    #[derive(Clone, Debug, serde::Serialize, WorthQuery)]
    #[worth_query(
        schema = ConflictSchema,
        context = GenericBoardContext,
        namespace = CONFLICT_NS,
        query = ConflictQuery,
        binding = ConflictBindingA,
        result = RowBinding,
        root = Task,
        explicit_shape,
        limits = 2,
        name = "conflict_query",
    )]
    pub struct ConflictReqA {
        #[worth_query(scope = ProjectKey)]
        pub project: u64,
    }

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct ConflictReqB {
        pub project: u64,
    }
    worth_query_structured_value_binding!(pub ConflictInputB for ConflictReqB {
        identity: worth_query_identity!(CONFLICT_NS, request)
    });
    fn project_of(req: ConflictReqB) -> u64 {
        req.project
    }
    worth_query_query_binding!(pub ConflictBindingB for ConflictReqB, schema ConflictSchema,
        context GenericBoardContext,
        identity worth_query_identity!(CONFLICT_NS, binding),
        input ConflictInputB,
        query ConflictQuery,
        scope ProjectKey, field ProjectKey::at("ProjectKey"), value project_of,
        limits results 9
    );

    pub struct ConflictExtra;
    impl Registration<ConflictSchema> for ConflictExtra {
        fn register(
            builder: ApplicationSchemaDeclarationBuilder<ConflictSchema>,
        ) -> Result<
            ApplicationSchemaDeclarationBuilder<ConflictSchema>,
            ApplicationSchemaDeclarationDenial,
        > {
            Ok(builder.application_query_binding::<ConflictBindingB>())
        }
    }

    fn conflict_def(
    ) -> ApplicationQueryDefinitionBuilder<ConflictSchema, ConflictQuery, (), TaskRow, Project>
    {
        let shape = ApplicationQueryResultShapeBuilder::<
            ConflictSchema,
            ConflictQuery,
            Task,
            TaskRow,
            RowBinding,
        >::new(entity(Board, Task))
        .build();
        ConflictQuery::definition_with(shape).root_path(
            ApplicationQueryRootPath::from(entity(Board, Project))
                .forward(relation(Board, Project, Contains)),
        )
    }

    worth_query_application_program! {
        pub program ConflictProgram for ConflictSchema {
            identity: "macros.parity.conflict-program.v1",
            contribution: ConflictContribution {
                identity: "macros.parity.conflict-contribution.v1",
                members: [Board, GenericBoardPrincipal, ConflictExtra],
            },
            feature: ConflictFeature => ConflictDef / ConflictRt {
                identity: "macros.parity.conflict-feature.v1",
                query(ConflictBindingA) { definition: conflict_def(); };
            }
        }
    }
}

#[test]
fn bound_application_query_still_refuses_duplicate_query_and_conflicting_binding() {
    let duplicate_def = || {
        DuplicateQuery::definition().root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
    };
    let dup_err = board()
        .bound_application_query::<DuplicateBinding>(duplicate_def())
        .bound_application_query::<DuplicateBinding>(duplicate_def())
        .build()
        .expect_err("duplicate query definition is still refused");
    assert!(
        matches!(dup_err, ApplicationSchemaDeclarationDenial::DuplicateMember),
        "unexpected error: {dup_err:?}"
    );
    let req = DuplicateReq { project: 1 };
    assert_eq!(req.project, 1);

    let conflict_decl = conflicting::ConflictSchema::declaration()
        .expect("conflicting bindings reach installation intact");
    let clock = WorthQueryAdmissionClock::system();
    let limits = WorthQueryApplicationLimits::bounded_serial(clock)
        .install()
        .expect("limits install");
    let validated = ApplicationProgramAuthoring::<
        conflicting::ConflictSchema,
        conflicting::ConflictProgram,
    >::begin()
    .with_schema(&conflict_decl)
    .validated_program()
    .expect("program validates");
    let install_err = in_memory_program(
        validated,
        conflict_decl,
        (conflicting::ConflictRt::new(),),
        ApplicationBuildIdentity::parse("7a7a7a7a-7a7a-7a7a-7a7a-7a7a7a7a7a7a").unwrap(),
        limits,
        |_, _, _| Ok(()),
    )
    .err()
    .expect("conflicting binding identity is refused at installation");
    let rendered = format!("{install_err:?}");
    assert!(
        rendered.contains("QueryInstallationDenied")
            && rendered.contains("macros.parity.conflict-query.v1.binding"),
        "expected QueryInstallationDenied for conflicting binding identity, got: {rendered}"
    );
    let _ = (
        conflicting::ConflictReqA { project: 1 },
        TaskRow::key::<DuplicateQuery>(),
    );
}
