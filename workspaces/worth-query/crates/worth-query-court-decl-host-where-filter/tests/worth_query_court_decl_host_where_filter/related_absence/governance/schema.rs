//! Minimal governed Daymark-like board schema with a real capability and grant
//! facts to prove fail-closed relation and target-field governance on
//! `where_no_related` queries through the public Host.
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};

pub mod access;
pub mod queries;
pub use access::*;
pub use queries::*;

#[derive(Clone, Copy)]
pub struct GovSchema;
impl Fragment for GovSchema {
    const NAME: &'static str = "related_absence_governed";
    type Bindings = NoBindings;
    type Members = (GovContribution,);
}
impl ApplicationSchema for GovSchema {
    const OWNER: &'static str = "worth.query.related-absence.gov";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for GovSchema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct GovContribution;
impl ApplicationSchemaContribution<GovSchema> for GovContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("related-absence.gov-contrib.v1");
    type Members = (GovModel, GovPrincipalBinding, GovAccess, GovQueries);
}

#[derive(Clone, Copy)]
pub struct GovModel;
impl Fragment for GovModel {
    const NAME: &'static str = "gov_model";
    type Bindings = GovBindings;
    type Members = (ExternalMapping, Principal, Project, Task, BoardGrant);
}
pub struct GovBindings;
impl BindingSet for GovBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for GovBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for GovBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for GovBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for GovBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}

macro_rules! gov_entity {
    ($name:ident, $aspects:ty, $relations:ty) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $aspects;
            type Relations = $relations;
        }
        impl $name {
            pub fn reference() -> ApplicationEntityRef<GovSchema, Self> {
                entity(GovModel, Self)
            }
        }
    };
}
gov_entity!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
gov_entity!(Principal, (PrincipalFacts,), (GrantGrantee, GrantGrantor));
gov_entity!(Project, (ProjectFacts,), (Contains,));
gov_entity!(Task, (TaskFacts,), (BlockedBy,));
gov_entity!(BoardGrant, (BoardGrantFacts,), (GrantResource, GrantParent));

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = (
        WorthQueryExternalPrincipalIdentity,
        WorthQueryPrincipalMappingStatus,
    );
}
impl
    Record<(
        WorthQueryExternalPrincipalIdentity,
        WorthQueryPrincipalMappingStatus,
    )> for GovBindings
{
    fn record() -> impl Shape<
        (
            WorthQueryExternalPrincipalIdentity,
            WorthQueryPrincipalMappingStatus,
        ),
        Self,
    > {
        field(
            "ExternalIdentityField",
            |r: &(WorthQueryExternalPrincipalIdentity, _)| r.0.clone(),
        )
        .equality_queryable(true)
        .and(
            "MappingStatus",
            |r: &(_, WorthQueryPrincipalMappingStatus)| r.1,
        )
        .equality_queryable(true)
        .build(|a, b, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>((a, b)))
    }
}
pub struct PrincipalFacts;
pub struct PrincipalRow(u64);
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
impl Record<PrincipalRow> for GovBindings {
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
pub struct ProjectRow(String);
impl Aspect for ProjectFacts {
    const NAME: &'static str = "ProjectFacts";
    type Row = ProjectRow;
}
impl Record<ProjectRow> for GovBindings {
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
impl Aspect for TaskFacts {
    const NAME: &'static str = "TaskFacts";
    type Row = (u64, String, String);
}
impl Record<(u64, String, String)> for GovBindings {
    fn record() -> impl Shape<(u64, String, String), Self> {
        field("TaskKey", |r: &(u64, String, String)| r.0)
            .equality_queryable(true)
            .read_only()
            .and("Status", |r: &(u64, String, String)| r.1.clone())
            .equality_queryable(true)
            .and("Gate", |r: &(u64, String, String)| r.2.clone())
            .equality_queryable(true)
            .build(|a, b, c, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>((a, b, c))
            })
    }
}
pub struct BoardGrantFacts;
type GrantTuple = (String, u64, u64, u64, String, u64, u64, u64, u64);
impl Aspect for BoardGrantFacts {
    const NAME: &'static str = "BoardGrantFacts";
    type Row = GrantTuple;
}
impl Record<GrantTuple> for GovBindings {
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

pub type ExtIdField = FieldSelection<
    GovModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MapStatus =
    FieldSelection<GovModel, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrinId = FieldSelection<GovModel, Principal, PrincipalFacts, u64>;
pub type ProjectKey = FieldSelection<GovModel, Project, ProjectFacts, String>;
pub type TaskKey = FieldSelection<GovModel, Task, TaskFacts, u64>;
pub type Status = FieldSelection<GovModel, Task, TaskFacts, String>;
pub type Gate = FieldSelection<GovModel, Task, TaskFacts, String>;
pub type GrantIdentity = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, String>;
pub type GrantAction = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantPurpose = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantStatus = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantScope = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, String>;
pub type GrantNotBefore = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantDepth = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;
pub type GrantDisclosure = FieldSelection<GovModel, BoardGrant, BoardGrantFacts, u64>;

macro_rules! gov_rel {
    ($name:ident, $from:ident -> $to:ident) => {
        pub struct $name;
        impl Relation for $name {
            const NAME: &'static str = stringify!($name);
            type To = $to;
        }
        impl $name {
            pub fn reference() -> ApplicationRelationRef<GovSchema, Self, $from, $to> {
                relation(GovModel, $from, Self)
            }
        }
    };
}
gov_rel!(MappingTarget, ExternalMapping -> Principal);
gov_rel!(Contains, Project -> Task);
gov_rel!(BlockedBy, Task -> Task);
gov_rel!(GrantGrantee, Principal -> BoardGrant);
gov_rel!(GrantGrantor, Principal -> BoardGrant);
gov_rel!(GrantResource, BoardGrant -> Project);
gov_rel!(GrantParent, BoardGrant -> BoardGrant);

pub struct GovPrincipalBinding;
impl GovPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        GovSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "GovPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExtIdField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MapStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrinId::at("PrincipalIdentity"),
                ),
            },
        )
    }
}
impl Registration<GovSchema> for GovPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<GovSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<GovSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}
