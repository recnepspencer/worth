//! Daymark-shaped schema with native String title-prefix and binary FocusWindow interval indexes.
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{
    application_schema::*, authentication::*, worth_query_native_scalar_bindings,
};
pub use worth_query_testkit_document_retention::facade::managed_probe_binding::{
    FocusWindow, FocusWindowProbeBinding,
};

#[derive(Clone, Copy)]
pub struct DaymarkSchema;
impl Fragment for DaymarkSchema {
    const NAME: &'static str = "OrderedIndexAdapterSchema";
    type Bindings = NoBindings;
    type Members = (super::Contribution,);
}
impl ApplicationSchema for DaymarkSchema {
    const OWNER: &'static str = "worth.tests.ordered_index_adapter";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DaymarkSchema {
    type Contributions = <Self as Fragment>::Members;
}

#[derive(Clone, Copy)]
pub struct Daymark;
impl Fragment for Daymark {
    const NAME: &'static str = "daymark";
    type Bindings = Codecs;
    type Members = (ExternalMapping, Principal, Workspace, TaskItem);
}
worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

macro_rules! entity_decl {
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
entity_decl!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
entity_decl!(Principal, (PrincipalFacts,), ());
entity_decl!(Workspace, (WorkspaceFacts,), (ContainsTask,));
entity_decl!(TaskItem, (TaskFacts,), ());

impl Workspace {
    pub fn reference() -> ApplicationEntityRef<DaymarkSchema, Self> {
        entity(Daymark, Self)
    }
}
impl TaskItem {
    pub fn reference() -> ApplicationEntityRef<DaymarkSchema, Self> {
        entity(Daymark, Self)
    }
}

pub struct ExternalIdentity;
pub struct ExternalIdentityRow {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
impl Record<ExternalIdentityRow> for Codecs {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |r: &ExternalIdentityRow| r.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { identity, status })
        })
    }
}

pub struct PrincipalFacts;
pub struct PrincipalFactsRow {
    identity: u64,
}
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsRow;
}
impl Record<PrincipalFactsRow> for Codecs {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        field("PrincipalIdentity", |r: &PrincipalFactsRow| r.identity)
            .equality_queryable(true)
            .read_only()
            .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsRow { identity })
            })
    }
}

pub struct WorkspaceFacts;
pub struct WorkspaceFactsRow {
    key: String,
}
impl Aspect for WorkspaceFacts {
    const NAME: &'static str = "WorkspaceFacts";
    type Row = WorkspaceFactsRow;
}
impl Record<WorkspaceFactsRow> for Codecs {
    fn record() -> impl Shape<WorkspaceFactsRow, Self> {
        field("WorkspaceKey", |r: &WorkspaceFactsRow| r.key.clone())
            .equality_queryable(true)
            .read_only()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(WorkspaceFactsRow { key })
            })
    }
}

pub struct TaskFacts;
pub struct TaskFactsRow {
    key: u64,
    title: String,
    start_min: i64,
    end_min: i64,
}
impl Aspect for TaskFacts {
    const NAME: &'static str = "TaskFacts";
    type Row = TaskFactsRow;
}
impl Record<TaskFactsRow> for Codecs {
    fn record() -> impl Shape<TaskFactsRow, Self> {
        field("TaskKey", |r: &TaskFactsRow| r.key)
            .equality_queryable(true)
            .and("TaskTitle", |r: &TaskFactsRow| r.title.clone())
            .and("StartMin", |r: &TaskFactsRow| r.start_min)
            .and("EndMin", |r: &TaskFactsRow| r.end_min)
            .build(
                |key, title, start_min, end_min, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(TaskFactsRow {
                        key,
                        title,
                        start_min,
                        end_min,
                    })
                },
            )
    }
}

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<DaymarkSchema, Self, ExternalMapping, Principal> {
        relation(Daymark, ExternalMapping, Self)
    }
}

pub struct ContainsTask;
impl Relation for ContainsTask {
    const NAME: &'static str = "ContainsTask";
    type To = TaskItem;
}
impl ContainsTask {
    pub fn reference() -> ApplicationRelationRef<DaymarkSchema, Self, Workspace, TaskItem> {
        relation(Daymark, Workspace, Self)
    }
}

pub type ExternalIdField =
    FieldSelection<Daymark, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type StatusField =
    FieldSelection<Daymark, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdField = FieldSelection<Daymark, Principal, PrincipalFacts, u64>;
pub type WorkspaceKeyField = FieldSelection<Daymark, Workspace, WorkspaceFacts, String>;
pub type TaskKeyField = FieldSelection<Daymark, TaskItem, TaskFacts, u64>;
pub type TaskTitleField = FieldSelection<Daymark, TaskItem, TaskFacts, String>;
pub type StartMinField = FieldSelection<Daymark, TaskItem, TaskFacts, i64>;
pub type EndMinField = FieldSelection<Daymark, TaskItem, TaskFacts, i64>;

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
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
                    ExternalIdField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    StatusField::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdField::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

pub struct TitlePrefixIndex;
impl ApplicationManagedIndexBinding<DaymarkSchema> for TitlePrefixIndex {
    type SourceBinding = super::queries::SourceBinding;
    type ProbeBinding = StringApplicationValueBinding;
    type Entity = TaskItem;
    const NAME: &'static str = "title_prefix_index";
    const PROVIDER: &'static str = "ordered-index-adapter.title-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}

pub struct FocusIntervalIndex;
impl ApplicationManagedIndexBinding<DaymarkSchema> for FocusIntervalIndex {
    type SourceBinding = super::queries::SourceBinding;
    type ProbeBinding = FocusWindowProbeBinding;
    type Entity = TaskItem;
    const NAME: &'static str = "focus_interval_index";
    const PROVIDER: &'static str = "ordered-index-adapter.focus-interval.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}
