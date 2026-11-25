//! Daymark-style schema, records, and `#[derive(WorthQueryRow)]` declarations.
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthQueryRecord, WorthQueryRelation, WorthQueryRow,
    WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const DAYMARK: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.v1");
pub const ROW_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.task-row.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.daymark.unified",
    name = "daymark_unified",
    members(super::program::DaymarkContribution)
)]
pub struct DaymarkSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "daymark",
    bindings = DaymarkCodecs,
    members(ExternalMapping, Principal, Project, Task)
)]
pub struct DaymarkFragment;

worth_query_native_scalar_bindings!(pub DaymarkCodecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for DaymarkCodecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for DaymarkCodecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(ExternalIdentity), relations(MappingTarget))]
pub struct ExternalMapping;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(PrincipalFacts))]
pub struct Principal;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(ProjectFacts),
    relations(Contains, NestsProject, WatchesProject),
    reference(DaymarkSchema, DaymarkFragment)
)]
pub struct Project;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(TaskFacts),
    relations(BlockedBy),
    reference(DaymarkSchema, DaymarkFragment)
)]
pub struct Task;

#[derive(WorthQuerySchema)]
#[worth_query(
    relation,
    to = Principal,
    from = ExternalMapping,
    reference(DaymarkSchema, DaymarkFragment)
)]
pub struct MappingTarget;

#[derive(WorthQueryRelation)]
#[worth_query(
    to = Task,
    from = Project,
    reference(DaymarkSchema, DaymarkFragment),
    acyclic,
    self_edges = false,
    unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge,
)]
pub struct Contains;

#[derive(WorthQueryRelation)]
#[worth_query(
    to = Task,
    from = Task,
    reference(DaymarkSchema, DaymarkFragment),
    acyclic,
    self_edges = false,
    unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge,
)]
pub struct BlockedBy;

/// Project nesting forbids self edges, like Daymark's project inboxes.
#[derive(WorthQueryRelation)]
#[worth_query(
    to = Project,
    from = Project,
    reference(DaymarkSchema, DaymarkFragment),
    acyclic,
    self_edges = false,
    unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge,
)]
pub struct NestsProject;

/// Each created project watches itself, like Daymark's project watch target.
#[derive(WorthQueryRelation)]
#[worth_query(to = Project, from = Project, reference(DaymarkSchema, DaymarkFragment))]
pub struct WatchesProject;

#[derive(WorthQueryRecord)]
#[worth_query(
    bindings = DaymarkCodecs,
    aspect = ExternalIdentity,
    fragment = DaymarkFragment,
    entity = ExternalMapping
)]
pub struct ExternalIdentityValue {
    #[worth_query(column = "ExternalIdentityField", equality)]
    pub identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(column = "MappingStatus", equality)]
    pub status: WorthQueryPrincipalMappingStatus,
}

#[derive(WorthQueryRecord)]
#[worth_query(
    bindings = DaymarkCodecs,
    aspect = PrincipalFacts,
    fragment = DaymarkFragment,
    entity = Principal
)]
pub struct PrincipalFactsValue {
    #[worth_query(column = "PrincipalIdentity", equality, read_only)]
    pub principal_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(
    bindings = DaymarkCodecs,
    aspect = ProjectFacts,
    fragment = DaymarkFragment,
    entity = Project
)]
pub struct ProjectFactsValue {
    #[worth_query(column = "ProjectKey", equality, unique, read_only)]
    pub project_key: u64,
    pub label: String,
}

#[derive(WorthQueryRecord)]
#[worth_query(
    bindings = DaymarkCodecs,
    aspect = TaskFacts,
    fragment = DaymarkFragment,
    entity = Task
)]
pub struct TaskFactsValue {
    #[worth_query(column = "TaskKey", equality, unique, read_only)]
    pub task_key: u64,
    pub title: String,
    #[worth_query(equality)]
    pub state: String,
    #[worth_query(equality)]
    pub priority: u64,
    #[worth_query(equality)]
    pub due_date: Option<CanonicalDate>,
}

/// Typed projected row usable directly as `result = TaskRow` on `#[derive(WorthQuery)]`.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DaymarkSchema, root = Task, namespace = ROW_NS)]
pub struct TaskRow {
    #[worth_query(field = TaskKey)]
    pub task_key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(field = State)]
    pub state: String,
    #[worth_query(field = Priority)]
    pub priority: u64,
    #[worth_query(field = DueDate)]
    pub due_date: Option<CanonicalDate>,
}

pub struct DaymarkPrincipal;
impl DaymarkPrincipal {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DaymarkPrincipal",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

impl Registration<DaymarkSchema> for DaymarkPrincipal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

worth_query_structured_value_binding!(pub UnitResultBinding for () {
    identity: worth_query_identity!(DAYMARK, result)
});
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(DAYMARK, denial)
});

pub struct DaymarkContext;
impl ApplicationBindingContext<DaymarkSchema> for DaymarkContext {
    type PrincipalBinding = DaymarkPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(DAYMARK, idempotency);
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
        DaymarkPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipal::reference()
    }
}
