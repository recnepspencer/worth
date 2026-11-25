//! Daymark-like task board schema with `Contains` (`Project -> Task`) and
//! `BlockedBy` (`Task -> Task`) relations for related-record absence/existence
//! courts and the seeded differential graph DST.
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};

#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "related_absence_board";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.related-absence";
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
        ApplicationSchemaContributionIdentity::new("related-absence.contribution.v1");
    type Members = (
        Board,
        PrincipalBinding,
        super::mutations::Registered,
        super::queries::Registered,
    );
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

#[derive(Clone, Copy)]
pub struct Board;
impl Fragment for Board {
    const NAME: &'static str = "board";
    type Bindings = BoardBindings;
    type Members = (ExternalMapping, Principal, Project, Task);
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
impl Scalar<i64> for BoardBindings {
    type Binding = I64ApplicationValueBinding;
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
entity!(Project, (ProjectFacts,), (Contains,));
entity!(Task, (TaskFacts,), (BlockedBy,));
impl Project {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Board, Self)
    }
}
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
    pub identity: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for BoardBindings {
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
    pub identity: u64,
}
impl Record<PrincipalFactsValue> for BoardBindings {
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
pub struct ProjectFacts;
impl Aspect for ProjectFacts {
    const NAME: &'static str = "ProjectFacts";
    type Row = ProjectFactsValue;
}
pub struct ProjectFactsValue {
    pub key: u64,
}
impl Record<ProjectFactsValue> for BoardBindings {
    fn record() -> impl Shape<ProjectFactsValue, Self> {
        field("ProjectKey", |row: &ProjectFactsValue| row.key)
            .equality_queryable(true)
            .read_only()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProjectFactsValue { key })
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
    pub notes: Option<String>,
    pub status: String,
    pub rank: i64,
    pub due: Option<CanonicalDate>,
}
impl Record<TaskFactsValue> for BoardBindings {
    fn record() -> impl Shape<TaskFactsValue, Self> {
        field("TaskKey", |row: &TaskFactsValue| row.key)
            .equality_queryable(true)
            .read_only()
            .and("Title", |row: &TaskFactsValue| row.title.clone())
            .and("Notes", |row: &TaskFactsValue| row.notes.clone())
            .and("Status", |row: &TaskFactsValue| row.status.clone())
            .equality_queryable(true)
            .and("Rank", |row: &TaskFactsValue| row.rank)
            .equality_queryable(true)
            .and("Due", |row: &TaskFactsValue| row.due)
            .build(
                |key, title, notes, status, rank, due, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(TaskFactsValue {
                        key,
                        title,
                        notes,
                        status,
                        rank,
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
pub type ProjectKey = FieldSelection<Board, Project, ProjectFacts, u64>;
pub type TaskKey = FieldSelection<Board, Task, TaskFacts, u64>;
pub type Title = FieldSelection<Board, Task, TaskFacts, String>;
pub type Notes = FieldSelection<Board, Task, TaskFacts, String>;
pub type Status = FieldSelection<Board, Task, TaskFacts, String>;
pub type Rank = FieldSelection<Board, Task, TaskFacts, i64>;
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
pub struct Contains;
impl Relation for Contains {
    const NAME: &'static str = "Contains";
    type To = Task;
}
impl Contains {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Project, Task> {
        relation(Board, Project, Self)
    }
}
pub struct BlockedBy;
impl Relation for BlockedBy {
    const NAME: &'static str = "BlockedBy";
    type To = Task;
}
impl BlockedBy {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Task, Task> {
        relation(Board, Task, Self)
    }
}
