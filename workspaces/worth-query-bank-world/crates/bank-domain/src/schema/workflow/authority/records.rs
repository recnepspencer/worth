//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use crate::schema::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct ApprovalRecords;
impl fragment::Fragment for ApprovalRecords {
    const NAME: &'static str = "approval_records";
    type Bindings = ApprovalRecordsBindings;
    type Members = (ApprovedBusinessPaymentGrant,);
}
pub struct ApprovalRecordsBindings;
impl fragment::BindingSet for ApprovalRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for ApprovalRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<PaymentId> for ApprovalRecordsBindings {
    type Binding = PaymentIdBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for ApprovalRecordsBindings {
    type Binding = worth_query_decl::facade::application_schema::U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct ApprovedBusinessPaymentGrant;
impl fragment::Entity for ApprovedBusinessPaymentGrant {
    const NAME: &'static str = "ApprovedBusinessPaymentGrant";
    type Aspects = (ApprovedBusinessPaymentGrantFacts,);
    type Relations = (
        ApprovedBusinessPaymentGrantResource,
        ApprovedBusinessPaymentGrantParent,
    );
}
impl ApprovedBusinessPaymentGrant {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(ApprovalRecords, Self)
    }
}
pub struct ApprovedBusinessPaymentGrantFacts;
impl fragment::Aspect for ApprovedBusinessPaymentGrantFacts {
    const NAME: &'static str = "ApprovedBusinessPaymentGrantFacts";
    type Row = ApprovedBusinessPaymentGrantFactsValue;
}
impl ApprovedBusinessPaymentGrantFacts {
    pub fn reference() -> AspectSelection<ApprovalRecords, ApprovedBusinessPaymentGrant, Self> {
        aspect(ApprovalRecords, ApprovedBusinessPaymentGrant, Self)
    }
}
pub struct ApprovedBusinessPaymentGrantFactsValue {
    pub approved_business_payment_grant_action: String,
    pub approved_business_payment_grant_purpose: String,
    pub approved_business_payment_grant_status: String,
    pub approved_business_payment_grant_workflow: PaymentId,
    pub approved_business_payment_grant_not_before: u64,
    pub approved_business_payment_grant_not_after: u64,
    pub approved_business_payment_grant_delegation_limit: u64,
}
impl fragment::Record<ApprovedBusinessPaymentGrantFactsValue> for ApprovalRecordsBindings {
    fn record() -> impl fragment::Shape<ApprovedBusinessPaymentGrantFactsValue, Self> {
        fragment::field(
            "ApprovedBusinessPaymentGrantAction",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_action.clone()
            },
        )
        .read_only()
        .and(
            "ApprovedBusinessPaymentGrantPurpose",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_purpose.clone()
            },
        )
        .read_only()
        .and(
            "ApprovedBusinessPaymentGrantStatus",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_status.clone()
            },
        )
        .and(
            "ApprovedBusinessPaymentGrantWorkflow",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_workflow
            },
        )
        .and(
            "ApprovedBusinessPaymentGrantNotBefore",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_not_before
            },
        )
        .and(
            "ApprovedBusinessPaymentGrantNotAfter",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_not_after
            },
        )
        .and(
            "ApprovedBusinessPaymentGrantDelegationLimit",
            |row: &ApprovedBusinessPaymentGrantFactsValue| {
                row.approved_business_payment_grant_delegation_limit
            },
        )
        .build(
            |approved_business_payment_grant_action,
             approved_business_payment_grant_purpose,
             approved_business_payment_grant_status,
             approved_business_payment_grant_workflow,
             approved_business_payment_grant_not_before,
             approved_business_payment_grant_not_after,
             approved_business_payment_grant_delegation_limit,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ApprovedBusinessPaymentGrantFactsValue {
                    approved_business_payment_grant_action,
                    approved_business_payment_grant_purpose,
                    approved_business_payment_grant_status,
                    approved_business_payment_grant_workflow,
                    approved_business_payment_grant_not_before,
                    approved_business_payment_grant_not_after,
                    approved_business_payment_grant_delegation_limit,
                })
            },
        )
    }
}
pub type ApprovedBusinessPaymentGrantAction = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    String,
>;
pub type ApprovedBusinessPaymentGrantPurpose = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    String,
>;
pub type ApprovedBusinessPaymentGrantStatus = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    String,
>;
pub type ApprovedBusinessPaymentGrantWorkflow = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    PaymentId,
>;
pub type ApprovedBusinessPaymentGrantNotBefore = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    u64,
>;
pub type ApprovedBusinessPaymentGrantNotAfter = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    u64,
>;
pub type ApprovedBusinessPaymentGrantDelegationLimit = FieldSelection<
    ApprovalRecords,
    ApprovedBusinessPaymentGrant,
    ApprovedBusinessPaymentGrantFacts,
    u64,
>;
pub struct ApprovedBusinessPaymentGrantResource;
impl fragment::Relation for ApprovedBusinessPaymentGrantResource {
    const NAME: &'static str = "ApprovedBusinessPaymentGrantResource";
    type To = PaymentIntent;
}
impl ApprovedBusinessPaymentGrantResource {
    pub fn reference(
    ) -> ApplicationRelationRef<BankSchema, Self, ApprovedBusinessPaymentGrant, PaymentIntent> {
        relation(ApprovalRecords, ApprovedBusinessPaymentGrant, Self)
    }
}
pub struct ApprovedBusinessPaymentGrantParent;
impl fragment::Relation for ApprovedBusinessPaymentGrantParent {
    const NAME: &'static str = "ApprovedBusinessPaymentGrantParent";
    type To = ApprovedBusinessPaymentGrant;
}
impl ApprovedBusinessPaymentGrantParent {
    pub fn reference() -> ApplicationRelationRef<
        BankSchema,
        Self,
        ApprovedBusinessPaymentGrant,
        ApprovedBusinessPaymentGrant,
    > {
        relation(ApprovalRecords, ApprovedBusinessPaymentGrant, Self)
    }
}
pub struct ApprovedBusinessPaymentGrantGrantor;
impl fragment::Relation for ApprovedBusinessPaymentGrantGrantor {
    const NAME: &'static str = "ApprovedBusinessPaymentGrantGrantor";
    type To = ApprovedBusinessPaymentGrant;
}
impl ApprovedBusinessPaymentGrantGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<BankSchema, Self, Principal, ApprovedBusinessPaymentGrant> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct ApprovedBusinessPaymentGrantGrantee;
impl fragment::Relation for ApprovedBusinessPaymentGrantGrantee {
    const NAME: &'static str = "ApprovedBusinessPaymentGrantGrantee";
    type To = ApprovedBusinessPaymentGrant;
}
impl ApprovedBusinessPaymentGrantGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<BankSchema, Self, Principal, ApprovedBusinessPaymentGrant> {
        relation(AccountRecords, Principal, Self)
    }
}
