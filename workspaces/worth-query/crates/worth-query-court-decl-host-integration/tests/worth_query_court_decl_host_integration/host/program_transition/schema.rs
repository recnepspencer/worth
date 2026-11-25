//! Daymark-shaped v1 / v2 / v3 schemas sharing the core `Project` / `Task`
//! fragment while v3 adds `TaskPriority::Priority` (equality-queryable) via
//! `Extend<Task, (Option<TaskPriority>,), ()>`.

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
use worth_query_host::facade::{
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

pub struct DaymarkBindings;
impl BindingSet for DaymarkBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for DaymarkBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for DaymarkBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for DaymarkBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for DaymarkBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}

pub struct DaymarkCore;
impl Fragment for DaymarkCore {
    const NAME: &'static str = "daymark_core";
    type Bindings = DaymarkBindings;
    type Members = (ExternalMapping, Principal, Project, Task);
}

pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for DaymarkBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityRow| {
            row.external_id.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityRow| row.status)
        .equality_queryable(true)
        .build(|external_id, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow {
                external_id,
                status,
            })
        })
    }
}

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}

pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = ();
}
pub struct PrincipalIdentity;
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityRow;
}
pub struct PrincipalIdentityRow {
    pub identity: u64,
}
impl Record<PrincipalIdentityRow> for DaymarkBindings {
    fn record() -> impl Shape<PrincipalIdentityRow, Self> {
        field("PrincipalIdentityField", |row: &PrincipalIdentityRow| {
            row.identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalIdentityRow { identity })
        })
    }
}

pub struct Project;
impl Entity for Project {
    const NAME: &'static str = "Project";
    type Aspects = (ProjectCore,);
    type Relations = (ProjectTasks,);
}
pub struct ProjectCore;
impl Aspect for ProjectCore {
    const NAME: &'static str = "ProjectCore";
    type Row = ProjectCoreRow;
}
pub struct ProjectCoreRow {
    pub key: String,
    pub title: String,
}
impl Record<ProjectCoreRow> for DaymarkBindings {
    fn record() -> impl Shape<ProjectCoreRow, Self> {
        field("ProjectKey", |row: &ProjectCoreRow| row.key.clone())
            .equality_queryable(true)
            .unique()
            .and("Title", |row: &ProjectCoreRow| row.title.clone())
            .build(|key, title, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProjectCoreRow { key, title })
            })
    }
}

pub struct ProjectTasks;
impl Relation for ProjectTasks {
    const NAME: &'static str = "ProjectTasks";
    type To = Task;
}

pub struct Task;
impl Entity for Task {
    const NAME: &'static str = "Task";
    type Aspects = (TaskCore,);
    type Relations = ();
}
pub struct TaskCore;
impl Aspect for TaskCore {
    const NAME: &'static str = "TaskCore";
    type Row = TaskCoreRow;
}
pub struct TaskCoreRow {
    pub task_key: String,
    pub v2_value: String,
}
impl Record<TaskCoreRow> for DaymarkBindings {
    fn record() -> impl Shape<TaskCoreRow, Self> {
        field("TaskKey", |row: &TaskCoreRow| row.task_key.clone())
            .equality_queryable(true)
            .unique()
            .and("V2Value", |row: &TaskCoreRow| row.v2_value.clone())
            .build(|task_key, v2_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(TaskCoreRow { task_key, v2_value })
            })
    }
}

/// Additive v3 extension adding `TaskPriority::Priority` (equality-indexed) to `Task`.
pub struct DaymarkV3Extensions;
impl Fragment for DaymarkV3Extensions {
    const NAME: &'static str = "daymark_v3_extensions";
    type Bindings = DaymarkBindings;
    type Members = (Extend<Task, (Option<TaskPriority>,), ()>,);
}
pub struct TaskPriority;
impl Aspect for TaskPriority {
    const NAME: &'static str = "TaskPriority";
    type Row = TaskPriorityRow;
}
pub struct TaskPriorityRow {
    pub priority: String,
}
impl Record<TaskPriorityRow> for DaymarkBindings {
    fn record() -> impl Shape<TaskPriorityRow, Self> {
        field("Priority", |row: &TaskPriorityRow| row.priority.clone())
            .equality_queryable(true)
            .build(|priority, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(TaskPriorityRow { priority })
            })
    }
}

pub type ProjectKeyField = FieldSelection<DaymarkCore, Project, ProjectCore, String>;
pub type ProjectTitleField = FieldSelection<DaymarkCore, Project, ProjectCore, String>;
pub type TaskKeyField = FieldSelection<DaymarkCore, Task, TaskCore, String>;
pub type TaskV2ValueField = FieldSelection<DaymarkCore, Task, TaskCore, String>;
pub type TaskPriorityField = FieldSelection<DaymarkV3Extensions, Task, TaskPriority, String>;

pub fn project_key_field() -> ProjectKeyField {
    ProjectKeyField::at("ProjectKey")
}
pub fn project_title_field() -> ProjectTitleField {
    ProjectTitleField::at("Title")
}
pub fn task_key_field() -> TaskKeyField {
    TaskKeyField::at("TaskKey")
}
pub fn task_v2_value_field() -> TaskV2ValueField {
    TaskV2ValueField::at("V2Value")
}
pub fn task_priority_field() -> TaskPriorityField {
    TaskPriorityField::at("Priority")
}

pub struct DaymarkPrincipalBinding<S>(std::marker::PhantomData<S>);
impl<S: ApplicationSchema> DaymarkPrincipalBinding<S> {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        S,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DaymarkPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    FieldSelection::<
                        DaymarkCore,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryExternalPrincipalIdentity,
                    >::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    FieldSelection::<
                        DaymarkCore,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryPrincipalMappingStatus,
                    >::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(fragment::relation(
                    DaymarkCore,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    FieldSelection::<DaymarkCore, Principal, PrincipalIdentity, u64>::at(
                        "PrincipalIdentityField",
                    ),
                ),
            },
        )
    }
}
impl<S: ApplicationSchema> Registration<S> for DaymarkPrincipalBinding<S> {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.principal_binding(Self::reference()))
    }
}

pub fn seed_base_project<S: ApplicationSchema>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<S>,
    schema: &WorthQueryInstalledApplicationSchema<S>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(DaymarkPrincipalBinding::<S>::reference())
        .unwrap();
    graph
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("daymark-principal").unwrap(),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                fragment::entity(DaymarkCore, Project),
                WorthQueryApplicationEntityKey::new("project-main").unwrap(),
            )
            .field(project_key_field(), "p-main".to_owned())
            .field(project_title_field(), "Daymark".to_owned()),
        )
        .unwrap();
    Ok(())
}
