use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewPayment;
use crate::model::PaymentId;
use crate::reads::PaymentSummary;
use crate::schema::{BankSchema, PaymentIntent};

use super::payment_summary_projection::{payment_summary_shape, project_payment_summary};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentDetailQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaymentDetailRequest {
    payment: PaymentId,
}

impl PaymentDetailRequest {
    pub const fn new(payment: PaymentId) -> Self {
        Self { payment }
    }

    pub const fn payment(self) -> PaymentId {
        self.payment
    }
}

pub const fn payment(payment: PaymentId) -> PaymentDetailRequest {
    PaymentDetailRequest::new(payment)
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub PaymentDetailQueryParametersBinding for PaymentDetailQueryParameters { identity: "PaymentDetailQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub PaymentDetailQueryResultBinding for PaymentSummary { identity: "PaymentSummary" });
worth_query_application_query!(
    pub PaymentDetailQuery for BankSchema,
    identity "PaymentDetailQuery",
    parameters PaymentDetailQueryParametersBinding,
    result PaymentDetailQueryResultBinding,
    scope PaymentIntent => "PaymentIntent",
    name "payment_detail"
);

pub fn payment_detail_definition() -> ApplicationQueryDefinition<
    BankSchema,
    PaymentDetailQuery,
    PaymentDetailQueryParameters,
    PaymentSummary,
    PaymentIntent,
> {
    ApplicationQueryDefinitionBuilder::declare(PaymentDetailQuery::reference())
        .root(PaymentIntent::reference())
        .scope(PaymentIntent::reference())
        .result_shape(
            payment_summary_shape::<
                PaymentDetailQuery,
                PaymentSummary,
                PaymentDetailQueryResultBinding,
            >()
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2, 6, 8))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewPayment::reference())
        .build()
        .expect("bank payment detail query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, PaymentDetailQuery> for PaymentSummary {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, PaymentDetailQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        project_payment_summary(row)
    }
}
