//! One record model owns field lowering and reusable scalar bindings.
#[path = "inspection_grant_key.rs"]
mod inspection_grant_key;
use super::*;
pub use inspection_grant_key::{InspectionGrantKey, InspectionGrantKeyBinding};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct InspectionGrants;
impl Fragment for InspectionGrants {
    const NAME: &'static str = "inspection_grants";
    type Bindings = InspectionGrantsBindings;
    type Members = (InspectionGrant,);
}
pub struct InspectionGrantsBindings;
impl BindingSet for InspectionGrantsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for InspectionGrantsBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<bool> for InspectionGrantsBindings {
    type Binding = BoolApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for InspectionGrantsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
pub struct InspectionGrant;
impl Entity for InspectionGrant {
    const NAME: &'static str = "InspectionGrant";
    type Aspects = (InspectionGrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl InspectionGrant {
    pub fn reference() -> ApplicationEntityRef<HostIdentitySchema, Self> {
        entity(InspectionGrants, Self)
    }
}
pub struct InspectionGrantFacts;
impl Aspect for InspectionGrantFacts {
    const NAME: &'static str = "InspectionGrantFacts";
    type Row = InspectionGrantFactsValue;
}
impl InspectionGrantFacts {
    pub fn reference() -> AspectSelection<InspectionGrants, InspectionGrant, Self> {
        aspect(InspectionGrants, InspectionGrant, Self)
    }
}
pub struct InspectionGrantFactsValue {
    pub grant_identity: InspectionGrantKey,
    pub grant_action: String,
    pub grant_purpose: String,
    pub grant_disclosure: String,
    pub grant_active: bool,
    pub grant_workflow: String,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_delegation_limit: u64,
}
impl Record<InspectionGrantFactsValue> for InspectionGrantsBindings {
    fn record() -> impl Shape<InspectionGrantFactsValue, Self> {
        field("GrantIdentity", |row: &InspectionGrantFactsValue| {
            row.grant_identity.clone()
        })
        .read_only()
        .and("GrantAction", |row: &InspectionGrantFactsValue| {
            row.grant_action.clone()
        })
        .read_only()
        .and("GrantPurpose", |row: &InspectionGrantFactsValue| {
            row.grant_purpose.clone()
        })
        .read_only()
        .and("GrantDisclosure", |row: &InspectionGrantFactsValue| {
            row.grant_disclosure.clone()
        })
        .read_only()
        .and("GrantActive", |row: &InspectionGrantFactsValue| {
            row.grant_active
        })
        .and("GrantWorkflow", |row: &InspectionGrantFactsValue| {
            row.grant_workflow.clone()
        })
        .and("GrantNotBefore", |row: &InspectionGrantFactsValue| {
            row.grant_not_before
        })
        .and("GrantNotAfter", |row: &InspectionGrantFactsValue| {
            row.grant_not_after
        })
        .and("GrantDelegationLimit", |row: &InspectionGrantFactsValue| {
            row.grant_delegation_limit
        })
        .build(
            |grant_identity,
             grant_action,
             grant_purpose,
             grant_disclosure,
             grant_active,
             grant_workflow,
             grant_not_before,
             grant_not_after,
             grant_delegation_limit,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(InspectionGrantFactsValue {
                    grant_identity,
                    grant_action,
                    grant_purpose,
                    grant_disclosure,
                    grant_active,
                    grant_workflow,
                    grant_not_before,
                    grant_not_after,
                    grant_delegation_limit,
                })
            },
        )
    }
}
pub type GrantIdentity =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, InspectionGrantKey>;
pub type GrantAction =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, String>;
pub type GrantPurpose =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, String>;
pub type GrantDisclosure =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, String>;
pub type GrantActive =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, bool>;
pub type GrantWorkflow =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, String>;
pub type GrantNotBefore =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, u64>;
pub type GrantNotAfter =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, u64>;
pub type GrantDelegationLimit =
    FieldSelection<InspectionGrants, InspectionGrant, InspectionGrantFacts, u64>;
pub struct Grantor;
impl Relation for Grantor {
    const NAME: &'static str = "Grantor";
    type To = InspectionGrant;
}
impl Grantor {
    pub fn reference(
    ) -> ApplicationRelationRef<HostIdentitySchema, Self, Principal, InspectionGrant> {
        relation(IdentityModel, Principal, Self)
    }
}
pub struct Grantee;
impl Relation for Grantee {
    const NAME: &'static str = "Grantee";
    type To = InspectionGrant;
}
impl Grantee {
    pub fn reference(
    ) -> ApplicationRelationRef<HostIdentitySchema, Self, Principal, InspectionGrant> {
        relation(IdentityModel, Principal, Self)
    }
}
pub struct GrantResource;
impl Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Account;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<HostIdentitySchema, Self, InspectionGrant, Account>
    {
        relation(InspectionGrants, InspectionGrant, Self)
    }
}
pub struct GrantParent;
impl Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = InspectionGrant;
}
impl GrantParent {
    pub fn reference(
    ) -> ApplicationRelationRef<HostIdentitySchema, Self, InspectionGrant, InspectionGrant> {
        relation(InspectionGrants, InspectionGrant, Self)
    }
}

impl fragment::Scalar<InspectionGrantKey> for InspectionGrantsBindings {
    type Binding = InspectionGrantKeyBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
