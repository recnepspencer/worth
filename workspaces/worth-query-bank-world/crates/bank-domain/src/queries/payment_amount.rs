//! A payment's amount as a single typed result, the operand of the
//! approved-payment workflow's limit condition.

use worth_foundational::expression_api::{ExpressionDenial, ExpressionType, ExpressionValue};
use worth_query_decl::facade::application_program::ApplicationExpressionOperandValue;
use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultShapeBuilder,
};
use worth_query_decl::facade::worth_query_application_query;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewPayment;
use crate::model::{Money, PaymentId, USD};
use crate::schema::{BankSchema, PaymentIntent};

use super::payment_summary_projection::payment_amount as amount_field;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentAmountQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentAmountRequest {
    payment: PaymentId,
}

impl PaymentAmountRequest {
    pub const fn new(payment: PaymentId) -> Self {
        Self { payment }
    }

    pub const fn payment(self) -> PaymentId {
        self.payment
    }
}

pub const fn payment_amount(payment: PaymentId) -> PaymentAmountRequest {
    PaymentAmountRequest::new(payment)
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub PaymentAmountQueryParametersBinding for PaymentAmountQueryParameters { identity: "PaymentAmountQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub PaymentAmountQueryResultBinding for Money<USD> { identity: "PaymentAmount" });
worth_query_application_query!(
    pub PaymentAmountQuery for BankSchema,
    identity "PaymentAmountQuery",
    parameters PaymentAmountQueryParametersBinding,
    result PaymentAmountQueryResultBinding,
    scope PaymentIntent => "PaymentIntent",
    name "payment_amount"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub PaymentAmountRequestBinding for PaymentAmountRequest { identity: "PaymentAmountRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub PaymentAmountQueryBinding for PaymentAmountRequest, schema BankSchema,
    identity "worth.bank.payment-amount-query-binding.v1",
    input PaymentAmountRequestBinding,
    query PaymentAmountQuery,
    parameters PaymentAmountQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result PaymentAmountQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope PaymentIntent, crate::schema::PaymentIdentity, crate::schema::PaymentIdentityField,
        PaymentId, worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::PaymentIdentityField::reference(),
    value PaymentAmountRequest::payment,
    limits results 1, work 1_000

);

pub fn payment_amount_definition() -> ApplicationQueryDefinition<
    BankSchema,
    PaymentAmountQuery,
    PaymentAmountQueryParameters,
    Money<USD>,
    PaymentIntent,
> {
    ApplicationQueryDefinitionBuilder::declare(PaymentAmountQuery::reference())
        .root(PaymentIntent::reference())
        .scope(PaymentIntent::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                BankSchema,
                PaymentAmountQuery,
                PaymentIntent,
                Money<USD>,
                PaymentAmountQueryResultBinding,
            >::new(PaymentIntent::reference())
            .field(amount_field())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewPayment::reference())
        .build()
        .expect("bank payment amount query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, PaymentAmountQuery> for Money<USD> {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, PaymentAmountQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(amount_field())
    }
}

/// An amount reads as its exact count of cents, so a condition compares
/// money without rounding.
impl ApplicationExpressionOperandValue for Money<USD> {
    fn expression_type() -> ExpressionType {
        ExpressionType::INT64
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        Ok(ExpressionValue::integer(i128::from(self.minor_units())))
    }
}
