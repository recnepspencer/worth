//! A small board whose native fields need no hand-written codecs, one
//! principal mapping, and the binding context every binding shares.
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};
use worth_query_decl::facade::{
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "single_source_board";
    type Bindings = NoBindings;
    type Members = (super::program::Contribution, super::program::Pinning);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.single-source";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = <Self as Fragment>::Members;
}

/// Shared identities: board-wide values derive from one root.
pub const BOARD: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("single-source.board.v1");

#[derive(Clone, Copy)]
pub struct Board;
impl Fragment for Board {
    const NAME: &'static str = "board";
    type Bindings = Codecs;
    type Members = (ExternalMapping, Principal, Task);
}
worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

macro_rules! entity {
    ($name:ident, $aspects:ty, $relations:ty) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $aspects;
            type Relations = $relations;
        }
    };
}
entity!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
entity!(Principal, (PrincipalFacts,), ());
entity!(Task, (TaskFacts,), ());
impl Task {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Board, Self)
    }
}

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
pub struct ExternalIdentityValue {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for Codecs {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &ExternalIdentityValue| row.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityValue { identity, status })
        })
    }
}
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
        field("PrincipalIdentity", |row: &PrincipalFactsValue| row.identity)
            .equality_queryable(true)
            .read_only()
            .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue { identity })
            })
    }
}
pub struct TaskFacts;
impl Aspect for TaskFacts {
    const NAME: &'static str = "TaskFacts";
    type Row = TaskFactsValue;
}
pub struct TaskFactsValue {
    pub key: u64,
    pub title: String,
    pub archived: bool,
    pub pinned: bool,
    pub due: Option<CanonicalDate>,
}
impl Record<TaskFactsValue> for Codecs {
    fn record() -> impl Shape<TaskFactsValue, Self> {
        field("TaskKey", |row: &TaskFactsValue| row.key)
            .equality_queryable(true)
            .read_only()
            .and("Title", |row: &TaskFactsValue| row.title.clone())
            .and("Archived", |row: &TaskFactsValue| row.archived)
            .and("Pinned", |row: &TaskFactsValue| row.pinned)
            .and("Due", |row: &TaskFactsValue| row.due)
            .build(
                |key, title, archived, pinned, due, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(TaskFactsValue {
                        key,
                        title,
                        archived,
                        pinned,
                        due,
                    })
                },
            )
    }
}

pub type ExternalIdentityField =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<Board, Principal, PrincipalFacts, u64>;
pub type TaskKey = FieldSelection<Board, Task, TaskFacts, u64>;
pub type Title = FieldSelection<Board, Task, TaskFacts, String>;
pub type Archived = FieldSelection<Board, Task, TaskFacts, bool>;
pub type Pinned = FieldSelection<Board, Task, TaskFacts, bool>;
pub type Due = FieldSelection<Board, Task, TaskFacts, CanonicalDate>;

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, ExternalMapping, Principal> {
        relation(Board, ExternalMapping, Self)
    }
}

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
