//! Host-level courts for Blocker 1 (fallible derived query `validate=` +
//! rejecting scalar parameter codec without panicking) and Blocker 3
//! (`#[serde(rename = "...")]` typed input field selector executing on Host).
use super::super::program::{day, scope, TASKS};
use super::super::schema::*;
use super::{GenericBoardContext, GenericBoardPrincipal};
use std::time::Duration;
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{entity, relation};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_decl::facade::{
    worth_query_identity_namespace, worth_query_structured_value_binding, WorthQuery,
    WorthQueryAction, WorthQueryProjection, WorthQuerySchema,
};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_host::facade::{
    worth_query_application_program, WorthQuerySourceCustody, WorthQuerySourceCustodyStep,
};

const NEG_ROW: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.negative-row.v1");
const GUARDED_QUERY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.guarded-query.v1");
const RENAMED_ACTION: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.renamed-wire-action.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "macros.parity.host-negative",
    name = "negative_board",
    members(NegativeContribution)
)]
pub struct NegativeSchema;

#[derive(Clone, Debug, PartialEq, Eq, WorthQueryProjection)]
#[worth_query(schema = NegativeSchema, root = Task, namespace = NEG_ROW)]
pub struct NegativeRow {
    #[worth_query(field = TaskKey)]
    pub key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(field = Notes)]
    pub notes: Option<String>,
}
worth_query_structured_value_binding!(pub NegativeRowBinding for NegativeRow {
    identity: "macros.parity.negative-row.v1.binding"
});

pub struct NonBlockedFilterBinding;
impl ApplicationScalarValueBinding for NonBlockedFilterBinding {
    type Value = String;
    type Unit = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Unit;
    type Decode = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Decode;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "macros.parity.non-blocked-filter.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;

    fn validate(value: &String) -> Result<(), ApplicationValueValidationDenial> {
        if value == "blocked" {
            return Err(ApplicationValueValidationDenial::rejected(
                WorthQueryPortableTypeIdentity::declared(Self::IDENTITY_NAME),
                "blocked filter term",
            ));
        }
        Ok(())
    }

    fn encode(value: &String) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        if value == "encode-fail" {
            return Err(ApplicationValueEncodeDenial::CodecRejected {
                binding_identity: WorthQueryPortableTypeIdentity::declared(Self::IDENTITY_NAME),
                reason: "codec rejected filter term",
            });
        }
        StringApplicationValueBinding::encode(value)
    }
}

impl ApplicationReadableScalarValueBinding for NonBlockedFilterBinding {
    fn decode(value: &AspectValue) -> Result<String, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value)
    }
}

fn reject_forbidden_input(input: &GuardedSearch) -> Result<(), &'static str> {
    if input.q == "forbidden" {
        return Err("forbidden search query");
    }
    Ok(())
}

#[derive(Clone, Debug, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = NegativeSchema,
    context = GenericBoardContext,
    namespace = GUARDED_QUERY,
    query = GuardedSearchQuery,
    binding = GuardedSearchBinding,
    result = NegativeRowBinding,
    root = Task,
    validate = reject_forbidden_input,
    limits = 16,
    name = "guarded_search",
)]
pub struct GuardedSearch {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
    #[worth_query(binding = NonBlockedFilterBinding)]
    pub q: String,
}

fn guarded_search_definition(
) -> ApplicationQueryDefinitionBuilder<NegativeSchema, GuardedSearchQuery, (), NegativeRow, Project>
{
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("title", TaskFacts::title())
        .and_then(|ops| ops.parameter("q", GuardedSearchQuery::q()))
        .and_then(|ops| ops.parse("contains_ignore_ascii_case(title, q)"))
        .expect("guarded filter admits");
    GuardedSearchQuery::definition()
        .root_path(
            ApplicationQueryRootPath::from(entity(Board, Project))
                .forward(relation(Board, Project, Contains)),
        )
        .order_by(
            NegativeRow::key(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .where_filter(filter)
}

fn default_title() -> String {
    String::new()
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[serde(rename_all = "snake_case")]
#[worth_query(
    schema = NegativeSchema,
    context = GenericBoardContext,
    namespace = RENAMED_ACTION,
    input = RenamedWireInput,
    operation = RenamedWireTask,
    binding = RenamedWireBinding,
    delivery = RenamedWireDone,
    body,
    program,
    reads(Task),
)]
#[allow(non_snake_case)]
pub struct RenameWireTitle {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    #[serde(
        alias = "legacy_title",
        default = "default_title",
        rename = "stored_title"
    )]
    pub new_title: String,
    pub fooBar: Option<String>,
    pub foo_bar: Option<String>,
}

fn renamed_wire_definition(
) -> ApplicationOperationDefinition<NegativeSchema, RenamedWireTask, RenameWireTitle> {
    let _ = default_title();
    RenamedWireTask::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), RenameWireTitle::new_title_input())
        .assign_optional_input(TaskFacts::notes(), RenameWireTitle::fooBar_input())
        .finish_pure_definition()
        .expect("renamed-wire body admits")
}

worth_query_application_program! {
    pub program NegativeProgram for NegativeSchema {
        identity: "macros.parity.negative-program.v1",
        contribution: NegativeContribution {
            identity: "macros.parity.negative-contribution.v1",
            members: [Board, GenericBoardPrincipal],
        },
        feature: NegativeFeature => NegativeDefinition / NegativeRuntime {
            identity: "macros.parity.negative-feature.v1",
            query(GuardedSearchBinding) { definition: guarded_search_definition(); };
            body_action(RenamedWireBinding, RenamedWireDone) as RenamedWireAction {
                operation: renamed_wire_definition();
            };
        }
    }
}

fn neg_key<E>(kind: &str, id: u64) -> WorthQueryApplicationEntityKey<NegativeSchema, E> {
    WorthQueryApplicationEntityKey::new(format!("{kind}-{id}")).unwrap()
}

fn seed_negative(
    graph: &mut WorthQueryPrimaryGraphBootstrap<NegativeSchema>,
    installed: &WorthQueryInstalledApplicationSchema<NegativeSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(GenericBoardPrincipal::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("board-owner").unwrap(),
        1_u64,
        WorthQueryExternalPrincipalIdentity::new("local://board", "owner").unwrap(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    for project in [1_u64, 2] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                entity(Board, Project),
                neg_key("project", project),
            )
            .field(ProjectKey::at("ProjectKey"), project),
        )?;
    }
    for (task, project, title, notes, due) in TASKS {
        let mut seed =
            WorthQueryApplicationEntitySeed::new(entity(Board, Task), neg_key("task", task))
                .field(TaskFacts::key(), task)
                .field(TaskFacts::title(), title.to_owned())
                .field(TaskFacts::done(), false);
        if let Some(notes) = notes {
            seed = seed.field(TaskFacts::notes(), notes.to_owned());
        }
        if let Some(due) = due {
            seed = seed.field(TaskFacts::due(), day(due));
        }
        graph.bind_entity(seed)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            relation(Board, Project, Contains),
            format!("contains-{task}"),
            neg_key("project", project),
            neg_key("task", task),
        ))?;
    }
    Ok(())
}

#[test]
fn derived_query_validate_and_rejecting_codec_return_typed_refusals_and_serde_rename_commits() {
    let clock = WorthQueryAdmissionClock::system();
    let declaration = NegativeSchema::declaration().expect("negative-court schema declares");
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .expect("limits install");
    let host = in_memory_program(
        ApplicationProgramAuthoring::<NegativeSchema, NegativeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .expect("program validates"),
        declaration,
        (NegativeRuntime::new(),),
        ApplicationBuildIdentity::parse("6a6a6a6a-6a6a-6a6a-6a6a-6a6a6a6a6a6a").unwrap(),
        limits,
        |_, graph, installed| seed_negative(graph, installed),
    )
    .expect("negative-court host installs");

    let req_scope = scope(&clock);
    let audience = || WorthQueryAuthenticationAudience::new("host").unwrap();
    let method = || WorthQueryAuthenticationMethod::new("local-process").unwrap();
    let local = WorthQueryLocalProcessPrincipal::new(
        "macros.local-owner.v1",
        WorthQueryExternalPrincipalIdentity::new("local://board", "owner").unwrap(),
        audience(),
        method(),
        Duration::from_secs(600),
    )
    .unwrap();
    let auth_owner = admit_authentication_adapter(
        host.runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method(), clock.clone()),
        local,
    )
    .unwrap()
    .authenticate_local(&req_scope)
    .unwrap();
    let request = host.request(&auth_owner, &req_scope);

    // Blocker 1a: `validate = reject_forbidden_input` refuses with typed InputValidation.
    let forbidden = request
        .query(GuardedSearch {
            project: 1,
            q: "forbidden".into(),
        })
        .execute()
        .err()
        .expect("validate= must reject before admission");
    assert_eq!(
        forbidden.kind(),
        WorthQueryApplicationRequestQueryDenialKind::InputPreparation
    );
    assert!(matches!(
        forbidden,
        WorthQueryApplicationRequestQueryDenial::InputPreparation(
            ApplicationQueryInputPreparationDenial::InputValidation(_)
        )
    ));

    // Blocker 1b: rejecting parameter codec returns typed ParameterEncoding without panicking.
    for bad in ["blocked", "encode-fail"] {
        let denied = request
            .query(GuardedSearch {
                project: 1,
                q: bad.into(),
            })
            .execute()
            .err()
            .expect("rejecting parameter codec must return typed denial, never panic");
        assert!(matches!(
            denied,
            WorthQueryApplicationRequestQueryDenial::InputPreparation(
                ApplicationQueryInputPreparationDenial::ParameterEncoding(_)
            )
        ));
    }

    // Valid search succeeds.
    let hits = request
        .query(GuardedSearch {
            project: 1,
            q: "Mill".into(),
        })
        .execute()
        .expect("valid guarded search executes")
        .rows()
        .to_vec();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Mill housing");

    // Blocker 3 & Fix 1: sibling `#[serde(alias, default, rename = "stored_title")]`
    // and Serde `rename_all = "snake_case"` preserving `fooBar` vs `foo_bar` commit cleanly!
    assert_eq!(RenameWireTitle::new_title_input().member(), "stored_title");
    assert_eq!(RenameWireTitle::fooBar_input().member(), "fooBar");
    assert_eq!(RenameWireTitle::foo_bar_input().member(), "foo_bar");
    let described = match request
        .mutate(RenameWireTitle {
            task: 1,
            new_title: "Milled via serde rename".into(),
            fooBar: Some("camel-preserved".into()),
            foo_bar: Some("snake-preserved".into()),
        })
        .without_source()
        .idempotency_owned(1_u64)
    {
        Ok(described) => described,
        Err((_, _, denial)) => panic!("serde-renamed input capture failed: {denial:?}"),
    };
    let step: WorthQuerySourceCustodyStep<
        NegativeSchema,
        RenameWireTitle,
        NegativeProgram,
        RenamedWireAction,
    > = WorthQuerySourceCustody::Described(described).resume(&host, &request);
    assert!(matches!(
        step,
        WorthQuerySourceCustodyStep::Committed { .. }
    ));
    let renamed = request
        .query(GuardedSearch {
            project: 1,
            q: "serde rename".into(),
        })
        .execute()
        .expect("renamed task is readable")
        .rows()
        .to_vec();
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0].title, "Milled via serde rename");
    assert_eq!(renamed[0].notes.as_deref(), Some("camel-preserved"));
}
