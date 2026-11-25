//! Real WORTH application schema, entities, records, relations, and principal binding.

use worth_query_decl::facade::{
    application_schema::{
        fragment::{self, *},
        *,
    },
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
};

use super::alias_shift_contract::{
    SetProjectAlias, SetProjectAliasBinding, WatchProjectByAliasCapability,
    WatchProjectByAliasOperation,
};
use super::capabilities::{
    ReadProjectCapability, ReadProjectOperation, SharedAccessContext, SharedAccessProvenance,
    SharedPrincipalBinding, WatchProjectCapability, WatchProjectOperation,
};
use super::mutations::{
    GrantProjectAccess, GrantProjectAccessBinding, RevokeProjectAccess, RevokeProjectAccessBinding,
    WriteProjectVersion, WriteProjectVersionBinding,
};
use super::owner_observe::{
    OwnerGrantQuery, OwnerGrantReadBinding, OwnerProjectQuery, OwnerProjectReadBinding,
};
use super::query::{ProjectStateQuery, ProjectStateReadBinding, ProjectUpdatedEffect};

pub struct SharedProjectSchema;

impl Fragment for SharedProjectSchema {
    const NAME: &'static str = "SharedProjectSchema";
    type Bindings = NoBindings;
    type Members = (SharedProjectSurface,);
}

impl ApplicationSchema for SharedProjectSchema {
    const OWNER: &'static str = "shared_access_court";
    const NAME: &'static str = <Self as Fragment>::NAME;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }

    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}

impl ApplicationSchemaComposition for SharedProjectSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct SharedProjectSurface;

impl ApplicationSchemaContribution<SharedProjectSchema> for SharedProjectSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.court.shared_access.surface.v1");
    type Members = (
        SharedModel,
        SharedPrincipalBinding,
        (
            SharedAccessContext,
            SharedAccessProvenance,
            ReadProjectOperation,
            ReadProjectCapability,
            WatchProjectOperation,
            WatchProjectCapability,
            WatchProjectByAliasOperation,
            WatchProjectByAliasCapability,
        ),
        (
            ProjectUpdatedEffect,
            ProjectStateQuery,
            ProjectStateReadBinding,
            OwnerProjectQuery,
            OwnerProjectReadBinding,
            OwnerGrantQuery,
            OwnerGrantReadBinding,
        ),
        (
            WriteProjectVersion,
            WriteProjectVersionBinding,
            GrantProjectAccess,
            GrantProjectAccessBinding,
            RevokeProjectAccess,
            RevokeProjectAccessBinding,
            SetProjectAlias,
            SetProjectAliasBinding,
        ),
    );
}

pub struct SharedModel;
impl Fragment for SharedModel {
    const NAME: &'static str = "shared_model";
    type Bindings = SharedModelBindings;
    type Members = (ExternalMapping, Principal, Project, ProjectGrant);
}

pub struct SharedModelBindings;
impl BindingSet for SharedModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}

impl Scalar<WorthQueryExternalPrincipalIdentity> for SharedModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for SharedModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for SharedModelBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for SharedModelBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<bool> for SharedModelBindings {
    type Binding = BoolApplicationValueBinding;
    const WRITABLE: bool = true;
}

pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentityAspect,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<SharedProjectSchema, Self> {
        entity(SharedModel, Self)
    }
}

pub struct ExternalIdentityAspect;
impl Aspect for ExternalIdentityAspect {
    const NAME: &'static str = "ExternalIdentityAspect";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for SharedModelBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.external_identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |r: &ExternalIdentityRow| {
            r.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityRow {
                    external_identity,
                    mapping_status,
                })
            },
        )
    }
}

pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentityAspect,);
    type Relations = (Grantor, Grantee);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<SharedProjectSchema, Self> {
        entity(SharedModel, Self)
    }
}

pub struct PrincipalIdentityAspect;
impl Aspect for PrincipalIdentityAspect {
    const NAME: &'static str = "PrincipalIdentityAspect";
    type Row = PrincipalIdentityRow;
}
pub struct PrincipalIdentityRow {
    pub principal_id: u64,
}
impl Record<PrincipalIdentityRow> for SharedModelBindings {
    fn record() -> impl Shape<PrincipalIdentityRow, Self> {
        field("PrincipalIdField", |r: &PrincipalIdentityRow| {
            r.principal_id
        })
        .equality_queryable(true)
        .read_only()
        .build(|principal_id, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalIdentityRow { principal_id })
        })
    }
}

pub struct Project;
impl Entity for Project {
    const NAME: &'static str = "Project";
    type Aspects = (ProjectStateAspect,);
    type Relations = (ProjectLiveSelf,);
}
impl Project {
    pub fn reference() -> ApplicationEntityRef<SharedProjectSchema, Self> {
        entity(SharedModel, Self)
    }
}

pub struct ProjectLiveSelf;
impl Relation for ProjectLiveSelf {
    const NAME: &'static str = "ProjectLiveSelf";
    type To = Project;
}
impl ProjectLiveSelf {
    pub fn reference() -> ApplicationRelationRef<SharedProjectSchema, Self, Project, Project> {
        relation(SharedModel, Project, Self)
    }
}

pub struct ProjectStateAspect;
impl Aspect for ProjectStateAspect {
    const NAME: &'static str = "ProjectStateAspect";
    type Row = ProjectStateRecord;
}
pub struct ProjectStateRecord {
    pub project_key: String,
    pub version: u64,
    pub project_alias: String,
}
impl Record<ProjectStateRecord> for SharedModelBindings {
    fn record() -> impl Shape<ProjectStateRecord, Self> {
        field("ProjectKey", |r: &ProjectStateRecord| r.project_key.clone())
            .equality_queryable(true)
            .read_only()
            .and("ProjectVersion", |r: &ProjectStateRecord| r.version)
            .and("ProjectAlias", |r: &ProjectStateRecord| {
                r.project_alias.clone()
            })
            .equality_queryable(true)
            .build(
                |project_key, version, project_alias, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(ProjectStateRecord {
                        project_key,
                        version,
                        project_alias,
                    })
                },
            )
    }
}

pub struct ProjectGrant;
impl Entity for ProjectGrant {
    const NAME: &'static str = "ProjectGrant";
    type Aspects = (ProjectGrantAspect,);
    type Relations = (GrantResource, GrantParent);
}
impl ProjectGrant {
    pub fn reference() -> ApplicationEntityRef<SharedProjectSchema, Self> {
        entity(SharedModel, Self)
    }
}

pub struct ProjectGrantAspect;
impl Aspect for ProjectGrantAspect {
    const NAME: &'static str = "ProjectGrantAspect";
    type Row = ProjectGrantRecord;
}
pub struct ProjectGrantRecord {
    pub grant_id: String,
    pub action: String,
    pub purpose: String,
    pub disclosure: String,
    pub mode: String,
    pub active: bool,
    pub workflow: String,
    pub not_before: u64,
    pub not_after: u64,
    pub delegation_limit: u64,
}
impl Record<ProjectGrantRecord> for SharedModelBindings {
    fn record() -> impl Shape<ProjectGrantRecord, Self> {
        field("GrantId", |r: &ProjectGrantRecord| r.grant_id.clone())
            .equality_queryable(true)
            .and("GrantAction", |r: &ProjectGrantRecord| r.action.clone())
            .and("GrantPurpose", |r: &ProjectGrantRecord| r.purpose.clone())
            .and("GrantDisclosure", |r: &ProjectGrantRecord| {
                r.disclosure.clone()
            })
            .and("GrantMode", |r: &ProjectGrantRecord| r.mode.clone())
            .and("GrantActive", |r: &ProjectGrantRecord| r.active)
            .and("GrantWorkflow", |r: &ProjectGrantRecord| r.workflow.clone())
            .and("GrantNotBefore", |r: &ProjectGrantRecord| r.not_before)
            .and("GrantNotAfter", |r: &ProjectGrantRecord| r.not_after)
            .and("GrantDelegationLimit", |r: &ProjectGrantRecord| {
                r.delegation_limit
            })
            .build(
                |grant_id,
                 action,
                 purpose,
                 disclosure,
                 mode,
                 active,
                 workflow,
                 not_before,
                 not_after,
                 delegation_limit,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(ProjectGrantRecord {
                        grant_id,
                        action,
                        purpose,
                        disclosure,
                        mode,
                        active,
                        workflow,
                        not_before,
                        not_after,
                        delegation_limit,
                    })
                },
            )
    }
}

pub type ExternalIdentityField = FieldSelection<
    SharedModel,
    ExternalMapping,
    ExternalIdentityAspect,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    SharedModel,
    ExternalMapping,
    ExternalIdentityAspect,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdField = FieldSelection<SharedModel, Principal, PrincipalIdentityAspect, u64>;
pub type ProjectKeyField = FieldSelection<SharedModel, Project, ProjectStateAspect, String>;
pub type ProjectVersionField = FieldSelection<SharedModel, Project, ProjectStateAspect, u64>;
pub type ProjectAliasField = FieldSelection<SharedModel, Project, ProjectStateAspect, String>;
pub type GrantIdField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantActionField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantPurposeField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantDisclosureField =
    FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantModeField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantActiveField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, bool>;
pub type GrantWorkflowField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, String>;
pub type GrantNotBeforeField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, u64>;
pub type GrantNotAfterField = FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, u64>;
pub type GrantDelegationLimitField =
    FieldSelection<SharedModel, ProjectGrant, ProjectGrantAspect, u64>;

pub use super::relations::{GrantParent, GrantResource, Grantee, Grantor, MappingTarget};
