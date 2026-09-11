use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryOrderingDirection, ApplicationQueryRootPath,
    },
    worth_query_application_query,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::{
    authorization::DiscoverOwnAccounts,
    model::CustomerRole,
    reads::PaymentSummary,
    schema::{
        AccountAuthorizedUser, AuthorizationAccount, AuthorizationRole, BankSchema, PaymentIntent,
        PaymentSource, PaymentStatus, PaymentStatusField, Principal,
    },
};

use super::payment_summary_projection::{
    payment_identity, payment_summary_shape, project_payment_summary,
};

pub struct PendingPaymentsQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingPaymentsRequest;

pub const fn pending_payments() -> PendingPaymentsRequest {
    PendingPaymentsRequest
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub PendingPaymentsQueryParametersBinding for PendingPaymentsQueryParameters { identity: "PendingPaymentsQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub PendingPaymentsQueryResultBinding for PaymentSummary { identity: "PaymentSummary" });
worth_query_application_query!(
    pub PendingPaymentsQuery for BankSchema,
    identity "PendingPaymentsQuery",
    parameters PendingPaymentsQueryParametersBinding,
    result PendingPaymentsQueryResultBinding,
    scope Principal => "Principal",
    name "pending_payments"
);

pub fn pending_payments_definition() -> ApplicationQueryDefinition<
    BankSchema,
    PendingPaymentsQuery,
    PendingPaymentsQueryParameters,
    PaymentSummary,
    Principal,
> {
    let identity = payment_identity();
    ApplicationQueryDefinitionBuilder::declare(PendingPaymentsQuery::reference())
        .root(PaymentIntent::reference())
        .scope(Principal::reference())
        .result_shape(
            payment_summary_shape::<
                PendingPaymentsQuery,
                PaymentSummary,
                PendingPaymentsQueryResultBinding,
            >()
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(3, 9, 8))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(DiscoverOwnAccounts::reference())
        .root_path(
            ApplicationQueryRootPath::from(Principal::reference())
                .forward(AccountAuthorizedUser::reference())
                .where_equal(
                    AuthorizationRole::reference(),
                    crate::schema::encoded_bank_value(CustomerRole::Approver),
                )
                .forward(AuthorizationAccount::reference())
                .reverse(PaymentSource::reference())
                .where_equal(
                    PaymentStatusField::reference(),
                    crate::schema::encoded_bank_value(PaymentStatus::ApprovalRequired),
                ),
        )
        .order_by(identity, ApplicationQueryOrderingDirection::Ascending)
        .build()
        .expect("bank pending payments query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, PendingPaymentsQuery> for PaymentSummary {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, PendingPaymentsQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        project_payment_summary(row)
    }
}
