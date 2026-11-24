//! The board's schema: derived markers and records, with the principal
//! record, principal binding and binding context written by hand beside them.
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::{application_schema::*, authentication::*};
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthQueryRecord, WorthQueryRelation, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.query.macros",
    name = "macros_board",
    members(super::program::Contribution)
)]
pub struct Schema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "board", bindings = Codecs,
    members(ExternalMapping, Principal, Project, Task))]
pub struct Board;
worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
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
    relations(Contains),
    reference(Schema, Board)
)]
pub struct Project;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(TaskFacts), reference(Schema, Board))]
pub struct Task;

#[derive(WorthQuerySchema)]
#[worth_query(relation, to = Principal, from = ExternalMapping, reference(Schema, Board))]
pub struct MappingTarget;
#[derive(WorthQueryRelation)]
#[worth_query(
    to = Task,
    from = Project,
    reference(Schema, Board),
    acyclic,
    self_edges = false,
    unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge,
)]
pub struct Contains;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = ExternalIdentity, fragment = Board,
    entity = ExternalMapping)]
pub struct ExternalIdentityValue {
    #[worth_query(column = "ExternalIdentityField", equality)]
    pub identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(column = "MappingStatus", equality)]
    pub status: WorthQueryPrincipalMappingStatus,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = ProjectFacts, fragment = Board, entity = Project)]
pub struct ProjectFactsValue {
    #[worth_query(column = "ProjectKey", equality, read_only)]
    pub key: u64,
}

/// Columns default to the field name in PascalCase; `TaskKey` keeps its own.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Board, entity = Task)]
pub struct TaskFactsValue {
    #[worth_query(column = "TaskKey", equality, read_only)]
    pub key: u64,
    pub title: String,
    pub notes: Option<String>,
    pub done: bool,
    pub due: Option<CanonicalDate>,
}

/// A hand-written aspect and record in the same fragment as derived ones.
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
pub struct PrincipalFactsValue {
    identity: u64,
}
impl Record<PrincipalFactsValue> for Codecs {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { identity })
        })
    }
}
pub type PrincipalIdentity = FieldSelection<Board, Principal, PrincipalFacts, u64>;

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
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
impl Registration<Schema> for PrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}

pub const BOARD: ApplicationIdentityNamespace = worth_query_identity_namespace!("macros.board.v1");
worth_query_structured_value_binding!(pub BodyResultBinding for () {
    identity: worth_query_identity!(BOARD, result)
});
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(BOARD, denial)
});

/// Every board binding authenticates, keys idempotency and denies alike.
pub struct BoardContext;
impl ApplicationBindingContext<Schema> for BoardContext {
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(BOARD, idempotency);
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}
