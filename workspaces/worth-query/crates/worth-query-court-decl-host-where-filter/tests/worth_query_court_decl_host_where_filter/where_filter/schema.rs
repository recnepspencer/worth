//! A small task board: projects own tasks; tasks carry searchable text,
//! an optional due date, a status, a rank and a unitless length.
use worth_foundational::facade::{CanonicalDate, CanonicalF64};
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};

#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "where_filter_board";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.where-filter";
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
        ApplicationSchemaContributionIdentity::new("where-filter.contribution.v1");
    type Members = (
        Board,
        PrincipalBinding,
        super::edit::EditTask,
        super::observe::ObserveSearch,
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
}
impl Scalar<CanonicalF64> for BoardBindings {
    type Binding = CanonicalF64ApplicationValueBinding;
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
entity!(Task, (TaskFacts,), ());
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
#[path = "schema/task.rs"]
mod task;
pub use task::*;

pub type ExternalIdentityField =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<Board, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<Board, Principal, PrincipalFacts, u64>;
pub type ProjectKey = FieldSelection<Board, Project, ProjectFacts, u64>;

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
