//! Real governed tree declarations for original-output inspection journeys.
use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryReference;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::bindings::DeniedDiagnosticFieldBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::bindings::DeniedDiagnosticInternalBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::bindings::DeniedDiagnosticRelationBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::bindings::DiagnosticAccountBinding;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::bindings::GovernedDiagnosticAccountBinding;

use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::activities;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::activity_identity;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::label;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::note;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::sequence;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::DiagnosticResult;
pub use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::DiagnosticResultBinding;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::DiagnosticUnitBinding;

worth_query_declaration::worth_query_structured_value_binding!(pub DiagnosticParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });

macro_rules! query {
    ($name:ident, $identity:literal) => {
        worth_query_declaration::worth_query_application_query!(
            pub $name for IdentityExecutionSchema, identity $identity,
            parameters DiagnosticParametersBinding, result DiagnosticResultBinding,
            scope Account => "Account", name $identity
        );
    };
}
query!(DiagnosticAccountQuery, "diagnostic_account");
query!(DeniedDiagnosticInternalQuery, "denied_diagnostic_internal");
query!(DeniedDiagnosticRelationQuery, "denied_diagnostic_relation");
query!(DeniedDiagnosticFieldQuery, "denied_diagnostic_field");

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DiagnosticPolicy {
    Complete,
    DenyInternal,
    DenyRelation,
    DenyField,
}

pub fn definition<Q>(
    reference: ApplicationQueryReference<
        IdentityExecutionSchema,
        Q,
        AccountSummaryParameters,
        DiagnosticResult,
        Account,
    >,
    policy: DiagnosticPolicy,
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    Q,
    AccountSummaryParameters,
    DiagnosticResult,
    Account,
>
where
    Q: ApplicationQueryMarkerIdentity<IdentityExecutionSchema> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = DiagnosticResult>,
{
    let child = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Q,
        Activity,
        (),
        DiagnosticUnitBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Q,
        Account,
        DiagnosticResult,
        Q::ResultBinding,
    >::new(Account::reference())
    .field(label())
    .optional_field(note())
    .relation(activities(), child)
    .build();
    let value = || {
        ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
            CapabilityDisclosure::AccountActivity,
        )
        .unwrap()
    };
    let all = ApplicationQueryInfluenceContract::permit_all;
    let without_explanation = || {
        ApplicationQueryInfluenceContract::permit(
            ApplicationQueryObservableInfluence::ALL
                .into_iter()
                .filter(|surface| *surface != ApplicationQueryObservableInfluence::Explanation),
        )
    };
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "diagnostic-account",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountStatus::at("AccountStatus"),
        value(),
        if policy == DiagnosticPolicy::DenyInternal {
            without_explanation()
        } else {
            all()
        },
    )
    .expect("record field contract")
    .use_field_by(ActivitySequence::at("ActivitySequence"), value(), all())
    .expect("record field contract")
    .disclose_field_by(
        label::<Q>(),
        value(),
        if policy == DiagnosticPolicy::DenyField {
            without_explanation()
        } else {
            all()
        },
    )
    .disclose_optional_field_by(note::<Q>(), value(), all())
    .disclose_field_by(activity_identity::<Q>(), value(), all())
    .disclose_field_by(sequence::<Q>(), value(), all())
    .disclose_relation_by(
        activities::<Q>(),
        value(),
        if policy == DiagnosticPolicy::DenyRelation {
            without_explanation()
        } else {
            all()
        },
    );
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter())
        .where_equal(AccountStatus::at("AccountStatus"), status_parameter())
        .order_by(sequence(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap()
}

pub fn governed_metadata(
) -> worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition<
    IdentityExecutionSchema,
    GovernedDiagnosticAccountBinding,
> {
    use worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition;
    use worth_query_declaration::facade::application_query::ApplicationQueryInspectionMetadataFacet;
    use worth_query_declaration::facade::application_query::ApplicationQueryInspectionMetadataRef;
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(
            ApplicationQueryDisclosureContract::governed_by(
                "diagnostic-metadata",
                TouchAccountCapability::reference(),
            ),
            |contract, facet| {
                contract.disclose_inspection_metadata_by(
                    ApplicationQueryInspectionMetadataRef::<
                        IdentityExecutionSchema,
                        GovernedDiagnosticAccountBinding,
                    >::new(facet),
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::AccountActivity,
                    )
                    .unwrap(),
                    ApplicationQueryInfluenceContract::permit([facet.influence()]),
                )
            },
        );
    ApplicationQueryInspectionDefinition::new(contract).unwrap()
}

#[path = "bindings/mod.rs"]
pub mod bindings;
#[path = "shape/mod.rs"]
pub mod shape;
