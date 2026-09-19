use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
        ApplicationQueryResultShapeBuilder, ExactlyOneResult, ForwardResultTraversal,
        OptionalOneResult, ReverseResultTraversal,
    },
    application_schema::{
        DeclaredApplicationUnit, EqualityPredicate, NoApplicationUnit, NoEqualityPredicate,
        ReadOnly, ReadWrite,
    },
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

use crate::model::{AccountId, BankPrincipalId, BusinessId, Money, PaymentId, USD};
use crate::reads::PaymentSummary;
use crate::schema::{
    Account, AccountIdentity, Approval, ApprovalPrincipal, BankSchema, Business, BusinessIdentity,
    BusinessIdentityField, PaymentAmount, PaymentApproval, PaymentBusiness, PaymentDestination,
    PaymentIdentity, PaymentIdentityField, PaymentInitiator, PaymentIntent, PaymentSource,
    PaymentState, PaymentStatus, PaymentStatusField, PaymentValue, Principal, PrincipalIdentity,
    PrincipalIdentityField, UsdCurrency,
};

pub(super) struct PaymentIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentIdentitySlot => "PaymentIdentitySlot");
struct PaymentAmountSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentAmountSlot => "PaymentAmountSlot");
struct PaymentStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentStatusSlot => "PaymentStatusSlot");
struct PaymentSourceSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentSourceSlot => "PaymentSourceSlot");
struct SourceIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(SourceIdentitySlot => "SourceIdentitySlot");
struct PaymentDestinationSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentDestinationSlot => "PaymentDestinationSlot");
struct DestinationIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(DestinationIdentitySlot => "DestinationIdentitySlot");
struct PaymentBusinessSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentBusinessSlot => "PaymentBusinessSlot");
struct BusinessIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(BusinessIdentitySlot => "BusinessIdentitySlot");
struct PaymentInitiatorSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentInitiatorSlot => "PaymentInitiatorSlot");
struct InitiatorIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(InitiatorIdentitySlot => "InitiatorIdentitySlot");
struct PaymentApprovalSlot;
worth_query_decl::facade::worth_query_portable_type!(PaymentApprovalSlot => "PaymentApprovalSlot");
struct ApprovalPrincipalSlot;
worth_query_decl::facade::worth_query_portable_type!(ApprovalPrincipalSlot => "ApprovalPrincipalSlot");
struct DecidingPrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(DecidingPrincipalIdentitySlot => "DecidingPrincipalIdentitySlot");

type PaymentIdentitySelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    PaymentIdentitySlot,
    BankSchema,
    PaymentIntent,
    PaymentIdentity,
    PaymentIdentityField,
    PaymentId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type PaymentAmountSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    PaymentAmountSlot,
    BankSchema,
    PaymentIntent,
    PaymentValue,
    PaymentAmount,
    Money<USD>,
    ReadWrite,
    NoEqualityPredicate,
    DeclaredApplicationUnit<UsdCurrency, USD>,
>;

type PaymentStatusSelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    PaymentStatusSlot,
    BankSchema,
    PaymentIntent,
    PaymentState,
    PaymentStatusField,
    PaymentStatus,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AccountIdentitySelector<Query, Slot> = ApplicationQueryResultFieldRef<
    Query,
    Slot,
    BankSchema,
    Account,
    crate::schema::Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type BusinessIdentitySelector<Query> = ApplicationQueryResultFieldRef<
    Query,
    BusinessIdentitySlot,
    BankSchema,
    Business,
    BusinessIdentity,
    BusinessIdentityField,
    BusinessId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type PrincipalIdentitySelector<Query, Slot> = ApplicationQueryResultFieldRef<
    Query,
    Slot,
    BankSchema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

pub(super) fn payment_summary_shape<Query, Result, ShapeBinding>(
) -> ApplicationQueryResultShapeBuilder<BankSchema, Query, PaymentIntent, Result, ShapeBinding>
where
    Query: worth_query_decl::facade::application_query::ApplicationQueryMarkerIdentity<BankSchema>,
    ShapeBinding: worth_query_decl::facade::application_schema::ApplicationStructuredValueBinding<
        Value = Result,
    >,
{
    let source = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Account,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Account::reference())
    .field(source_identity());
    let destination = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Account,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Account::reference())
    .field(destination_identity());
    let business = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Business,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Business::reference())
    .field(business_identity());
    let initiator = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(initiator_identity());
    let deciding_principal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(deciding_principal_identity());
    let approval = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        Query,
        Approval,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Approval::reference())
    .relation(approval_principal(), deciding_principal);
    ApplicationQueryResultShapeBuilder::new(PaymentIntent::reference())
        .field(payment_identity())
        .field(payment_amount())
        .field(payment_status())
        .relation(payment_source(), source)
        .relation(payment_destination(), destination)
        .relation(payment_business(), business)
        .relation(payment_initiator(), initiator)
        .relation(payment_approval(), approval)
}

pub(super) fn project_payment_summary<Query>(
    row: &WorthQueryApplicationProjectionRow<'_, BankSchema, Query>,
) -> Result<PaymentSummary, WorthQueryApplicationProjectionDenial>
where
    Query: worth_query_decl::facade::application_query::ApplicationQueryMarkerIdentity<BankSchema>,
{
    let deciding_principal = row
        .optional(payment_approval())?
        .map(|approval| {
            approval
                .one(approval_principal())?
                .field(deciding_principal_identity())
        })
        .transpose()?;
    Ok(PaymentSummary::from_projection(
        row.field(payment_identity())?,
        row.one(payment_business())?.field(business_identity())?,
        row.one(payment_source())?.field(source_identity())?,
        row.one(payment_destination())?
            .field(destination_identity())?,
        row.one(payment_initiator())?.field(initiator_identity())?,
        row.field(payment_amount())?,
        row.field(payment_status())?,
        deciding_principal,
    ))
}

pub(super) fn payment_identity<Query>() -> PaymentIdentitySelector<Query> {
    ApplicationQueryResultFieldRef::new("payment", PaymentIdentityField::reference())
}

fn payment_amount<Query>() -> PaymentAmountSelector<Query> {
    ApplicationQueryResultFieldRef::new("amount", PaymentAmount::reference())
}

fn payment_status<Query>() -> PaymentStatusSelector<Query> {
    ApplicationQueryResultFieldRef::new("status", PaymentStatusField::reference())
}

fn source_identity<Query>() -> AccountIdentitySelector<Query, SourceIdentitySlot> {
    ApplicationQueryResultFieldRef::new("source", AccountIdentity::reference())
}

fn destination_identity<Query>() -> AccountIdentitySelector<Query, DestinationIdentitySlot> {
    ApplicationQueryResultFieldRef::new("destination", AccountIdentity::reference())
}

fn business_identity<Query>() -> BusinessIdentitySelector<Query> {
    ApplicationQueryResultFieldRef::new("business", BusinessIdentityField::reference())
}

fn initiator_identity<Query>() -> PrincipalIdentitySelector<Query, InitiatorIdentitySlot> {
    ApplicationQueryResultFieldRef::new("initiator", PrincipalIdentityField::reference())
}

fn deciding_principal_identity<Query>(
) -> PrincipalIdentitySelector<Query, DecidingPrincipalIdentitySlot> {
    ApplicationQueryResultFieldRef::new("deciding_principal", PrincipalIdentityField::reference())
}

fn payment_source<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    PaymentSourceSlot,
    BankSchema,
    PaymentSource,
    PaymentIntent,
    Account,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("source", PaymentSource::reference())
}

fn payment_destination<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    PaymentDestinationSlot,
    BankSchema,
    PaymentDestination,
    PaymentIntent,
    Account,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("destination", PaymentDestination::reference())
}

fn payment_business<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    PaymentBusinessSlot,
    BankSchema,
    PaymentBusiness,
    PaymentIntent,
    Business,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("business", PaymentBusiness::reference())
}

fn payment_initiator<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    PaymentInitiatorSlot,
    BankSchema,
    PaymentInitiator,
    Principal,
    PaymentIntent,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("initiator", PaymentInitiator::reference())
}

fn payment_approval<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    PaymentApprovalSlot,
    BankSchema,
    PaymentApproval,
    PaymentIntent,
    Approval,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("approval", PaymentApproval::reference())
}

fn approval_principal<Query>() -> ApplicationQueryResultRelationRef<
    Query,
    ApprovalPrincipalSlot,
    BankSchema,
    ApprovalPrincipal,
    Approval,
    Principal,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one(
        "deciding_principal",
        ApprovalPrincipal::reference(),
    )
}
