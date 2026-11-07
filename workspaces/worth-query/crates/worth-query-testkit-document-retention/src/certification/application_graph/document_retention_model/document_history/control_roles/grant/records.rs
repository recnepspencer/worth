//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment::*;
#[derive(Clone, Copy)]
pub struct ControlGrants;
impl Fragment for ControlGrants {
    const NAME: &'static str = "control_grants";
    type Bindings = ControlGrantsBindings;
    type Members = (ControlGrant,);
}
pub struct ControlGrantsBindings;
impl BindingSet for ControlGrantsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for ControlGrantsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl Scalar<WorkflowGrantKey> for ControlGrantsBindings {
    type Binding = WorkflowGrantKeyBinding;
}
impl Scalar<u64> for ControlGrantsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct ControlGrant;
impl Entity for ControlGrant {
    const NAME: &'static str = "ControlGrant";
    type Aspects = (ControlGrantFacts,);
    type Relations = (ControlGrantResource, ControlGrantParent);
}
impl ControlGrant {
    pub fn reference() -> ApplicationEntityRef<DocumentRetentionSchema, Self> {
        entity(ControlGrants, Self)
    }
}
pub struct ControlGrantFacts;
impl Aspect for ControlGrantFacts {
    const NAME: &'static str = "ControlGrantFacts";
    type Row = ControlGrantFactsValue;
}
impl ControlGrantFacts {
    pub fn reference() -> AspectSelection<ControlGrants, ControlGrant, Self> {
        aspect(ControlGrants, ControlGrant, Self)
    }
}
pub struct ControlGrantFactsValue {
    pub control_grant_action_field: String,
    pub control_grant_purpose_field: String,
    pub control_grant_status_field: String,
    pub control_grant_workflow_field: WorkflowGrantKey,
    pub control_grant_not_before_field: u64,
    pub control_grant_not_after_field: u64,
    pub control_grant_delegation_limit_field: u64,
}
impl Record<ControlGrantFactsValue> for ControlGrantsBindings {
    fn record() -> impl Shape<ControlGrantFactsValue, Self> {
        field("ControlGrantActionField", |row: &ControlGrantFactsValue| {
            row.control_grant_action_field.clone()
        })
        .read_only()
        .and(
            "ControlGrantPurposeField",
            |row: &ControlGrantFactsValue| row.control_grant_purpose_field.clone(),
        )
        .read_only()
        .and("ControlGrantStatusField", |row: &ControlGrantFactsValue| {
            row.control_grant_status_field.clone()
        })
        .and(
            "ControlGrantWorkflowField",
            |row: &ControlGrantFactsValue| row.control_grant_workflow_field.clone(),
        )
        .and(
            "ControlGrantNotBeforeField",
            |row: &ControlGrantFactsValue| row.control_grant_not_before_field,
        )
        .and(
            "ControlGrantNotAfterField",
            |row: &ControlGrantFactsValue| row.control_grant_not_after_field,
        )
        .and(
            "ControlGrantDelegationLimitField",
            |row: &ControlGrantFactsValue| row.control_grant_delegation_limit_field,
        )
        .build(
            |control_grant_action_field,
             control_grant_purpose_field,
             control_grant_status_field,
             control_grant_workflow_field,
             control_grant_not_before_field,
             control_grant_not_after_field,
             control_grant_delegation_limit_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ControlGrantFactsValue {
                    control_grant_action_field,
                    control_grant_purpose_field,
                    control_grant_status_field,
                    control_grant_workflow_field,
                    control_grant_not_before_field,
                    control_grant_not_after_field,
                    control_grant_delegation_limit_field,
                })
            },
        )
    }
}
pub type ControlGrantActionField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, String>;
pub type ControlGrantPurposeField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, String>;
pub type ControlGrantStatusField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, String>;
pub type ControlGrantWorkflowField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, WorkflowGrantKey>;
pub type ControlGrantNotBeforeField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, u64>;
pub type ControlGrantNotAfterField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, u64>;
pub type ControlGrantDelegationLimitField =
    FieldSelection<ControlGrants, ControlGrant, ControlGrantFacts, u64>;
pub struct ControlGrantResource;
impl Relation for ControlGrantResource {
    const NAME: &'static str = "ControlGrantResource";
    type To = WorkflowAuthoringGrant;
}
impl ControlGrantResource {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, ControlGrant, WorkflowAuthoringGrant>
    {
        relation(ControlGrants, ControlGrant, Self)
    }
}
pub struct ControlGrantParent;
impl Relation for ControlGrantParent {
    const NAME: &'static str = "ControlGrantParent";
    type To = ControlGrant;
}
impl ControlGrantParent {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, ControlGrant, ControlGrant> {
        relation(ControlGrants, ControlGrant, Self)
    }
}
pub struct ControlGrantGrantor;
impl Relation for ControlGrantGrantor {
    const NAME: &'static str = "ControlGrantGrantor";
    type To = ControlGrant;
}
impl ControlGrantGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, Principal, ControlGrant> {
        relation(
            super::super::super::super::schema::RetentionModel,
            Principal,
            Self,
        )
    }
}
pub struct ControlGrantGrantee;
impl Relation for ControlGrantGrantee {
    const NAME: &'static str = "ControlGrantGrantee";
    type To = ControlGrant;
}
impl ControlGrantGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<DocumentRetentionSchema, Self, Principal, ControlGrant> {
        relation(
            super::super::super::super::schema::RetentionModel,
            Principal,
            Self,
        )
    }
}
