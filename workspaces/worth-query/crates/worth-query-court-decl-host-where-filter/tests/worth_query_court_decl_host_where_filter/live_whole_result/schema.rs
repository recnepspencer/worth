//! Daymark-like task board schema for whole-result live subscription courts
//! and 64-seed Differential Live-Subscription DST.
//!
//! Includes `Project`, `Task` (with optional `Due` and unexamined `Notes`),
//! `Contains`, `BlockedBy`, and a real capability `BoardGrant` so both public
//! and governed whole-result live subscriptions, grant revocation/restoration,
//! principal disablement, and Directory Home reopen are exercised honestly.
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};

#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "live_whole_result_board";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.live-whole-result";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl Schema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("live-whole-result.contribution.v1");
    type Members = (
        Board,
        PrincipalBinding,
        super::access::AccessRegistration,
        super::mutations::Registered,
        super::queries::Registered,
    );
}

#[derive(Clone, Copy)]
pub struct Board;
impl Fragment for Board {
    const NAME: &'static str = "board";
    type Bindings = BoardBindings;
    type Members = (ExternalMapping, Principal, Project, Task, BoardGrant);
}
pub struct BoardBindings;
impl BindingSet for BoardBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for BoardBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for BoardBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for BoardBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for BoardBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<CanonicalDate> for BoardBindings {
    type Binding = CanonicalDateApplicationValueBinding;
    const WRITABLE: bool = true;
}

macro_rules! board_entity {
    ($name:ident, $aspects:ty, $relations:ty) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $aspects;
            type Relations = $relations;
        }
        impl $name {
            pub fn reference() -> ApplicationEntityRef<Schema, Self> {
                entity(Board, Self)
            }
        }
    };
}
board_entity!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
board_entity!(Principal, (PrincipalFacts,), (GrantGrantee, GrantGrantor));
board_entity!(Project, (ProjectFacts,), (Contains,));
board_entity!(Task, (TaskFacts,), (BlockedBy,));
board_entity!(BoardGrant, (BoardGrantFacts,), (GrantResource, GrantParent));

pub struct ExternalIdentity;
pub type ExternalIdentityRow = (
    WorthQueryExternalPrincipalIdentity,
    WorthQueryPrincipalMappingStatus,
);
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
impl Record<ExternalIdentityRow> for BoardBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.0.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |r: &ExternalIdentityRow| r.1)
        .equality_queryable(true)
        .build(|a, b, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>((a, b)))
    }
}

pub struct PrincipalFacts;
pub struct PrincipalRow(pub u64);
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
impl Record<PrincipalRow> for BoardBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalIdentity", |r: &PrincipalRow| r.0)
            .equality_queryable(true)
            .read_only()
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow(a))
            })
    }
}

pub struct ProjectFacts;
pub struct ProjectRow(pub String);
impl Aspect for ProjectFacts {
    const NAME: &'static str = "ProjectFacts";
    type Row = ProjectRow;
}
impl Record<ProjectRow> for BoardBindings {
    fn record() -> impl Shape<ProjectRow, Self> {
        field("ProjectKey", |r: &ProjectRow| r.0.clone())
            .equality_queryable(true)
            .read_only()
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProjectRow(a))
            })
    }
}

pub struct TaskFacts;
pub struct TaskFactsValue {
    pub key: u64,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub due: Option<CanonicalDate>,
}
impl Aspect for TaskFacts {
    const NAME: &'static str = "TaskFacts";
    type Row = TaskFactsValue;
}
impl Record<TaskFactsValue> for BoardBindings {
    fn record() -> impl Shape<TaskFactsValue, Self> {
        field("TaskKey", |r: &TaskFactsValue| r.key)
            .equality_queryable(true)
            .read_only()
            .and("Title", |r: &TaskFactsValue| r.title.clone())
            .and("Notes", |r: &TaskFactsValue| r.notes.clone())
            .and("Status", |r: &TaskFactsValue| r.status.clone())
            .equality_queryable(true)
            .and("Due", |r: &TaskFactsValue| r.due)
            .build(
                |key, title, notes, status, due, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(TaskFactsValue {
                        key,
                        title,
                        notes,
                        status,
                        due,
                    })
                },
            )
    }
}

pub struct BoardGrantFacts;
pub type GrantTuple = (String, u64, u64, u64, String, u64, u64, u64, u64);
impl Aspect for BoardGrantFacts {
    const NAME: &'static str = "BoardGrantFacts";
    type Row = GrantTuple;
}
impl Record<GrantTuple> for BoardBindings {
    fn record() -> impl Shape<GrantTuple, Self> {
        field("GrantIdentity", |r: &GrantTuple| r.0.clone())
            .equality_queryable(true)
            .and("GrantAction", |r: &GrantTuple| r.1)
            .equality_queryable(true)
            .read_only()
            .and("GrantPurpose", |r: &GrantTuple| r.2)
            .equality_queryable(true)
            .read_only()
            .and("GrantStatus", |r: &GrantTuple| r.3)
            .equality_queryable(true)
            .and("GrantScope", |r: &GrantTuple| r.4.clone())
            .equality_queryable(true)
            .and("GrantNotBefore", |r: &GrantTuple| r.5)
            .equality_queryable(true)
            .read_only()
            .and("GrantNotAfter", |r: &GrantTuple| r.6)
            .equality_queryable(true)
            .read_only()
            .and("GrantDepth", |r: &GrantTuple| r.7)
            .equality_queryable(true)
            .read_only()
            .and("GrantDisclosure", |r: &GrantTuple| r.8)
            .equality_queryable(true)
            .read_only()
            .build(
                |a, b, c, d, e, f, g, h, i, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>((a, b, c, d, e, f, g, h, i))
                },
            )
    }
}

pub type ExternalIdentityField =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<Board, Principal, PrincipalFacts, u64>;
pub type ProjectKey = FieldSelection<Board, Project, ProjectFacts, String>;
pub type TaskKey = FieldSelection<Board, Task, TaskFacts, u64>;
pub type Title = FieldSelection<Board, Task, TaskFacts, String>;
pub type Notes = FieldSelection<Board, Task, TaskFacts, String>;
pub type Status = FieldSelection<Board, Task, TaskFacts, String>;
pub type Due = FieldSelection<Board, Task, TaskFacts, CanonicalDate>;
pub type GrantIdentity = FieldSelection<Board, BoardGrant, BoardGrantFacts, String>;
pub type GrantAction = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantPurpose = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantStatus = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantScope = FieldSelection<Board, BoardGrant, BoardGrantFacts, String>;
pub type GrantNotBefore = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantDepth = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;
pub type GrantDisclosure = FieldSelection<Board, BoardGrant, BoardGrantFacts, u64>;

macro_rules! board_rel {
    ($name:ident, $from:ident -> $to:ident) => {
        pub struct $name;
        impl Relation for $name {
            const NAME: &'static str = stringify!($name);
            type To = $to;
        }
        impl $name {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, $from, $to> {
                relation(Board, $from, Self)
            }
        }
    };
}
board_rel!(MappingTarget, ExternalMapping -> Principal);
board_rel!(Contains, Project -> Task);
board_rel!(BlockedBy, Task -> Task);
board_rel!(GrantGrantee, Principal -> BoardGrant);
board_rel!(GrantGrantor, Principal -> BoardGrant);
board_rel!(GrantResource, BoardGrant -> Project);
board_rel!(GrantParent, BoardGrant -> BoardGrant);

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
