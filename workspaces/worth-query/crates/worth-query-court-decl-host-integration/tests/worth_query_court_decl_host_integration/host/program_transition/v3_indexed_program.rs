//! Daymark v3 successor declaring a stock `text_prefix` managed index over the
//! retained `TaskCore::V2Value` field, plus its declarative `v2 -> v3` transition.

use super::schema::*;
use super::v2_program::{DaymarkV2Program, DaymarkV2Schema};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_program::*,
    application_query::{ApplicationQueryOrderingDirection, ApplicationQueryRootPath},
    application_schema::{
        fragment::{self, *},
        *,
    },
    worth_query_identity_namespace, worth_query_program_transition,
    worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryRow,
};
use worth_query_host::facade::{
    application_installation::*, primary_graph::SignalConditionalEvaluationBudget,
    runtime::WorthQueryApplicationQueryResourceProfile, worth_query_application_program,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::application_runtime_surface as identity;

const ROW_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.v3.indexed.row");
const SOURCE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.v3.indexed.source");
const PREFIX_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.v3.indexed.prefix");

// Column-named selections the derives resolve as `<Selection>::at("<Column>")`.
type ProjectKey = ProjectKeyField;
type TaskKey = TaskKeyField;
type V2Value = TaskV2ValueField;
type Priority = TaskPriorityField;

pub struct DaymarkV3IndexedSchema;
impl Fragment for DaymarkV3IndexedSchema {
    const NAME: &'static str = "DaymarkSchema";
    type Bindings = NoBindings;
    type Members = (DaymarkV3IndexedSurface,);
}
impl ApplicationSchema for DaymarkV3IndexedSchema {
    const OWNER: &'static str = "daymark_transition_court";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DaymarkV3IndexedSchema {
    type Contributions = <Self as Fragment>::Members;
}

worth_query_structured_value_binding!(pub IndexedUnitBinding for () {
    identity: "daymark.v3.indexed.unit.v1"
});

pub struct IndexedContext;
impl ApplicationBindingContext<DaymarkV3IndexedSchema> for IndexedContext {
    type PrincipalBinding = DaymarkPrincipalBinding<DaymarkV3IndexedSchema>;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = IndexedUnitBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "daymark.v3.indexed.idempotency.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkV3IndexedSchema,
        DaymarkPrincipalBinding<DaymarkV3IndexedSchema>,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DaymarkV3IndexedSchema, root = Task, namespace = ROW_NS)]
pub struct IndexedTaskRow {
    #[worth_query(field = TaskKey)]
    pub task_key: String,
    #[worth_query(field = V2Value)]
    pub v2_value: String,
    #[worth_query(field = Priority)]
    pub priority: String,
}

/// The managed index's Source: every retained task of one project.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkV3IndexedSchema, context = IndexedContext, namespace = SOURCE_NS,
    result = IndexedTaskRow, root = Task, scope = ProjectKey, limits = 64
)]
pub struct IndexedTaskSource {
    #[worth_query(scope)]
    pub project: String,
}

/// Stock ASCII case-insensitive prefix index over the retained `V2Value` text.
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = DaymarkV3IndexedSchema,
    record = Task,
    source = IndexedTaskSourceBinding,
    probe = String,
    kind = text_prefix(IndexedTaskRow::v2_value()),
    namespace = "daymark.v3.value_prefix",
    version = 1,
)]
pub struct ValuePrefixIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkV3IndexedSchema, context = IndexedContext, namespace = PREFIX_NS,
    scope = ProjectKey, managed_index = ValuePrefixIndex, limits = 64
)]
pub struct FindTasksByValuePrefix {
    #[worth_query(scope)]
    pub project: String,
    #[worth_query(probe)]
    pub prefix: String,
}

worth_query_application_program! {
    pub program DaymarkV3IndexedProgram for DaymarkV3IndexedSchema {
        identity: "daymark.program.v3.indexed",
        contribution: DaymarkV3IndexedSurface {
            identity: "daymark.transition.surface.v3.indexed",
            members: [
                DaymarkCore,
                DaymarkV3Extensions,
                DaymarkPrincipalBinding<DaymarkV3IndexedSchema>,
            ],
        },
        managed_indexes: [ValuePrefixIndex],
        feature: DaymarkV3IndexedFeature => DaymarkV3IndexedDefinition / DaymarkV3IndexedRuntime {
            identity: "daymark.v3.indexed.feature.v1",
            query(IndexedTaskSourceBinding) {
                definition: IndexedTaskSourceQuery::definition()
                    .root_path(
                        ApplicationQueryRootPath::from(fragment::entity(DaymarkCore, Project))
                            .forward(fragment::relation(DaymarkCore, Project, ProjectTasks)),
                    )
                    .order_by(IndexedTaskRow::task_key(), ApplicationQueryOrderingDirection::Ascending);
            };
            query(FindTasksByValuePrefixBinding) {
                definition: FindTasksByValuePrefixQuery::definition()
                    .order_by(IndexedTaskRow::task_key(), ApplicationQueryOrderingDirection::Ascending);
            };
        }
    }
}

pub fn v3_indexed_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
) -> WorthQueryApplicationHomeRequest<'static, DaymarkV3IndexedSchema, DaymarkV3IndexedProgram> {
    let declaration = DaymarkV3IndexedSchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<DaymarkV3IndexedSchema, DaymarkV3IndexedProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (DaymarkV3IndexedRuntime::new(),),
        build,
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, schema| seed_base_project(graph, schema),
    )
}

worth_query_program_transition! {
    /// In-place upgrade whose successor declares a stock managed index.
    pub struct DaymarkV2ToV3Indexed;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3IndexedSchema, DaymarkV3IndexedProgram, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_priority_field()
            => default = "Normal".to_owned(), required = false, label = "Priority::Normal",
    ],
    rebuild_indexes = [task_priority_field()],
    description = "Add TaskPriority::Priority and a V2Value text-prefix managed index",
}
