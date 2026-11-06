//! One record model owns field lowering and reusable scalar bindings.
#[path = "workflow_grant_key.rs"]
mod workflow_grant_key;
use super::*;
pub use workflow_grant_key::{WorkflowGrantKey, WorkflowGrantKeyBinding};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct WorkflowGrants;
impl Fragment for WorkflowGrants {
    const NAME: &'static str = "workflow_grants";
    type Bindings = WorkflowGrantsBindings;
    type Members = (WorkflowAuthoringGrant,);
}
pub struct WorkflowGrantsBindings;
impl BindingSet for WorkflowGrantsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for WorkflowGrantsBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for WorkflowGrantsBindings {
    type Binding =
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct WorkflowAuthoringGrant;
impl Entity for WorkflowAuthoringGrant {
    const NAME: &'static str = "WorkflowAuthoringGrant";
    type Aspects = (WorkflowAuthoringGrantFacts,);
    type Relations = (
        WorkflowGrantResource,
        WorkflowGrantRelated,
        WorkflowGrantParent,
    );
}
impl WorkflowAuthoringGrant {
    pub fn reference() -> ApplicationEntityRef<DocumentRetentionSchema, Self> {
        entity(WorkflowGrants, Self)
    }
}
pub struct WorkflowAuthoringGrantFacts;
impl Aspect for WorkflowAuthoringGrantFacts {
    const NAME: &'static str = "WorkflowAuthoringGrantFacts";
    type Row = WorkflowAuthoringGrantFactsValue;
}
impl WorkflowAuthoringGrantFacts {
    pub fn reference() -> AspectSelection<WorkflowGrants, WorkflowAuthoringGrant, Self> {
        aspect(WorkflowGrants, WorkflowAuthoringGrant, Self)
    }
}
pub struct WorkflowAuthoringGrantFactsValue {
    pub workflow_grant_identity_field: WorkflowGrantKey,
    pub workflow_grant_action_field: String,
    pub workflow_grant_purpose_field: String,
    pub workflow_grant_status_field: String,
    pub workflow_grant_workflow_field: String,
    pub workflow_grant_not_before_field: u64,
    pub workflow_grant_not_after_field: u64,
    pub workflow_grant_delegation_limit_field: u64,
}
impl Record<WorkflowAuthoringGrantFactsValue> for WorkflowGrantsBindings {
    fn record() -> impl Shape<WorkflowAuthoringGrantFactsValue, Self> {
        field(
            "WorkflowGrantIdentityField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_identity_field.clone(),
        )
        .read_only()
        .and(
            "WorkflowGrantActionField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_action_field.clone(),
        )
        .read_only()
        .and(
            "WorkflowGrantPurposeField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_purpose_field.clone(),
        )
        .read_only()
        .and(
            "WorkflowGrantStatusField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_status_field.clone(),
        )
        .and(
            "WorkflowGrantWorkflowField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_workflow_field.clone(),
        )
        .and(
            "WorkflowGrantNotBeforeField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_not_before_field,
        )
        .and(
            "WorkflowGrantNotAfterField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_not_after_field,
        )
        .and(
            "WorkflowGrantDelegationLimitField",
            |row: &WorkflowAuthoringGrantFactsValue| row.workflow_grant_delegation_limit_field,
        )
        .build(
            |workflow_grant_identity_field,
             workflow_grant_action_field,
             workflow_grant_purpose_field,
             workflow_grant_status_field,
             workflow_grant_workflow_field,
             workflow_grant_not_before_field,
             workflow_grant_not_after_field,
             workflow_grant_delegation_limit_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(WorkflowAuthoringGrantFactsValue {
                    workflow_grant_identity_field,
                    workflow_grant_action_field,
                    workflow_grant_purpose_field,
                    workflow_grant_status_field,
                    workflow_grant_workflow_field,
                    workflow_grant_not_before_field,
                    workflow_grant_not_after_field,
                    workflow_grant_delegation_limit_field,
                })
            },
        )
    }
}
pub type WorkflowGrantIdentityField = FieldSelection<
    WorkflowGrants,
    WorkflowAuthoringGrant,
    WorkflowAuthoringGrantFacts,
    WorkflowGrantKey,
>;
pub type WorkflowGrantActionField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, String>;
pub type WorkflowGrantPurposeField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, String>;
pub type WorkflowGrantStatusField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, String>;
pub type WorkflowGrantWorkflowField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, String>;
pub type WorkflowGrantNotBeforeField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, u64>;
pub type WorkflowGrantNotAfterField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, u64>;
pub type WorkflowGrantDelegationLimitField =
    FieldSelection<WorkflowGrants, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, u64>;
pub struct WorkflowDocumentOwner;
impl Relation for WorkflowDocumentOwner {
    const NAME: &'static str = "WorkflowDocumentOwner";
    type To = Document;
}
impl WorkflowDocumentOwner {
    pub fn reference() -> ApplicationRelationRef<DocumentRetentionSchema, Self, Principal, Document>
    {
        relation(super::super::super::schema::RetentionModel, Principal, Self)
    }
}
pub struct WorkflowGrantResource;
impl Relation for WorkflowGrantResource {
    const NAME: &'static str = "WorkflowGrantResource";
    type To = Document;
}
impl WorkflowGrantResource {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, WorkflowAuthoringGrant, Document>
    {
        relation(WorkflowGrants, WorkflowAuthoringGrant, Self)
    }
}
pub struct WorkflowGrantRelated;
impl Relation for WorkflowGrantRelated {
    const NAME: &'static str = "WorkflowGrantRelated";
    type To = Document;
}
impl WorkflowGrantRelated {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, WorkflowAuthoringGrant, Document>
    {
        relation(WorkflowGrants, WorkflowAuthoringGrant, Self)
    }
}
pub struct WorkflowGrantParent;
impl Relation for WorkflowGrantParent {
    const NAME: &'static str = "WorkflowGrantParent";
    type To = WorkflowAuthoringGrant;
}
impl WorkflowGrantParent {
    pub fn reference() -> ApplicationRelationRef<
        DocumentRetentionSchema,
        Self,
        WorkflowAuthoringGrant,
        WorkflowAuthoringGrant,
    > {
        relation(WorkflowGrants, WorkflowAuthoringGrant, Self)
    }
}
pub struct WorkflowGrantGrantor;
impl Relation for WorkflowGrantGrantor {
    const NAME: &'static str = "WorkflowGrantGrantor";
    type To = WorkflowAuthoringGrant;
}
impl WorkflowGrantGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, Principal, WorkflowAuthoringGrant>
    {
        relation(super::super::super::schema::RetentionModel, Principal, Self)
    }
}
pub struct WorkflowGrantGrantee;
impl Relation for WorkflowGrantGrantee {
    const NAME: &'static str = "WorkflowGrantGrantee";
    type To = WorkflowAuthoringGrant;
}
impl WorkflowGrantGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, Principal, WorkflowAuthoringGrant>
    {
        relation(super::super::super::schema::RetentionModel, Principal, Self)
    }
}

impl fragment::Scalar<WorkflowGrantKey> for WorkflowGrantsBindings {
    type Binding = WorkflowGrantKeyBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
