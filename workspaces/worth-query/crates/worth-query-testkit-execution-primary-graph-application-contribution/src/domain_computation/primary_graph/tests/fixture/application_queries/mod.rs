use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountMembershipTag;
use crate::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountSecondaryActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use crate::domain_computation::primary_graph::tests::fixture::ViewAccount;
use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryReference;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryRootPath;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::AccountSummaryQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::AccountSummaryQueryParametersBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::AccountSummaryQueryResultBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::CrossRootQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::CrossRootQueryResultBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::GovernedAccountSummaryQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::OrderedAccountSummaryQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::ScopedAccountSummaryQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::markers::ScopedAccountSummaryQueryResultBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::public_membership_definition;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::PublicAccountMembershipQuery;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::PublicAccountMembershipResult;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::public_scoped::public_scoped_definition;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::public_scoped::PublicScopedAccountSummaryQuery;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::activity_sequence_result_field;
pub use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
pub struct AccountSummaryParameters;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSummaryResult {
    pub status: String,
    pub label: String,
}
pub struct StatusParameter;
pub struct StatusResultSlot;
pub struct LabelResultSlot;
pub struct ActivitySequenceResultSlot;
worth_query_declaration::worth_query_portable_type!(
    AccountSummaryResult => "worth.query.test.execution.account_summary.result.v1"
);
worth_query_declaration::worth_query_portable_type!(
    StatusResultSlot => "worth.query.test.execution.account_summary.status_slot.v1"
);
worth_query_declaration::worth_query_portable_type!(
    LabelResultSlot => "worth.query.test.execution.account_summary.label_slot.v1"
);
worth_query_declaration::worth_query_portable_type!(
    ActivitySequenceResultSlot => "worth.query.test.execution.activity_sequence.slot.v1"
);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivitySequenceResult {
    pub sequence: u64,
}
worth_query_declaration::worth_query_portable_type!(
    ActivitySequenceResult => "worth.query.test.execution.activity_sequence.result.v1"
);

impl ActivitySequenceResult {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl AccountSummaryResult {
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl<
        Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
            IdentityExecutionSchema,
        >,
    >
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        Query,
    > for AccountSummaryResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            Query,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        Ok(Self {
            status: row.field(status_result_field::<Query>())?,
            label: row.field(label_result_field::<Query>())?,
        })
    }
}

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        CrossRootQuery,
    > for ActivitySequenceResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            CrossRootQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        Ok(Self {
            sequence: row.field(activity_sequence_result_field())?,
        })
    }
}

pub fn status_result_field<Query>(
) -> ApplicationQueryResultFieldRef<Query, StatusResultSlot, IdentityExecutionSchema, AccountStatus>
{
    ApplicationQueryResultFieldRef::new("status", AccountStatus::at("AccountStatus"))
}

pub fn label_result_field<Query>(
) -> ApplicationQueryResultFieldRef<Query, LabelResultSlot, IdentityExecutionSchema, AccountLabel> {
    ApplicationQueryResultFieldRef::new("label", AccountLabel::at("AccountLabel"))
}

pub fn cross_root_definition(
    status: &str,
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    CrossRootQuery,
    AccountSummaryParameters,
    ActivitySequenceResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        CrossRootQuery,
        Activity,
        ActivitySequenceResult,
        CrossRootQueryResultBinding,
    >::new(Activity::reference())
    .field(activity_sequence_result_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(CrossRootQuery::reference())
        .root(Activity::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccount::reference())
        .root_path(
            ApplicationQueryRootPath::from(Account::reference())
                .where_equal(AccountStatus::at("AccountStatus"), encoded_string(status))
                .expect("record field contract")
                .forward(AccountPrimaryActivity::reference()),
        )
        .root_path(
            ApplicationQueryRootPath::from(Account::reference())
                .where_equal(AccountStatus::at("AccountStatus"), encoded_string(status))
                .expect("record field contract")
                .forward(AccountSecondaryActivity::reference()),
        )
        .root_path(
            ApplicationQueryRootPath::from(Account::reference())
                .where_equal(AccountStatus::at("AccountStatus"), encoded_string(status))
                .expect("record field contract")
                .forward(AccountAllActivity::reference()),
        )
        .order_by(
            activity_sequence_result_field(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap()
}

pub fn definition<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
            IdentityExecutionSchema,
        > + 'static,
>(
    reference: ApplicationQueryReference<
        IdentityExecutionSchema,
        Query,
        AccountSummaryParameters,
        AccountSummaryResult,
        Account,
    >,
    disclosure: ApplicationQueryDisclosureContract,
    ordered: bool,
    requires_view: bool,
    advanced_read_lanes: bool,
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    Query,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
>
where
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = AccountSummaryResult>,
{
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Query,
        Account,
        AccountSummaryResult,
        Query::ResultBinding,
    >::new(Account::reference())
    .field(status_result_field::<Query>())
    .field(label_result_field::<Query>())
    .build();
    let basis_support = if advanced_read_lanes {
        ApplicationQueryBasisSupport::current_and_pinned().with_preview()
    } else {
        ApplicationQueryBasisSupport::current_and_pinned()
    };
    let lanes = if advanced_read_lanes {
        ApplicationQueryLaneEligibility::one_shot()
            .with_historical()
            .with_preview()
    } else {
        ApplicationQueryLaneEligibility::one_shot()
    };
    let builder = if requires_view {
        ApplicationQueryDefinitionBuilder::declare(reference)
            .root(Account::reference())
            .scope(Account::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(disclosure)
            .basis_support(basis_support)
            .lanes(lanes)
            .requires_ability(ViewAccount::reference())
    } else {
        ApplicationQueryDefinitionBuilder::declare(reference)
            .root(Account::reference())
            .scope(Account::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(disclosure)
            .basis_support(basis_support)
            .lanes(lanes)
            .public()
    }
    .parameter(status_parameter())
    .where_equal(AccountStatus::at("AccountStatus"), status_parameter());
    let builder = if ordered {
        builder.order_by(
            label_result_field::<Query>(),
            ApplicationQueryOrderingDirection::Ascending,
        )
    } else {
        builder
    };
    builder.build().unwrap()
}

pub type EncodedDisclosure = ApplicationEncodedScalarValue<CapabilityDisclosureBinding>;
pub type EncodedString = ApplicationEncodedScalarValue<StringApplicationValueBinding>;

pub fn encoded_disclosure(value: CapabilityDisclosure) -> EncodedDisclosure {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}

pub fn encoded_string(value: &str) -> EncodedString {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).expect("fixture string must encode")
}

#[path = "account_summary/mod.rs"]
pub mod account_summary;
#[path = "markers/mod.rs"]
pub mod markers;
#[path = "parameter_reference/mod.rs"]
pub mod parameter_reference;
#[path = "public_membership/mod.rs"]
pub mod public_membership;
#[path = "public_scoped/mod.rs"]
pub mod public_scoped;

pub use account_summary::{
    account_summary_definition, governed_account_summary_definition,
    ordered_account_summary_definition, scoped_account_summary_definition,
};
