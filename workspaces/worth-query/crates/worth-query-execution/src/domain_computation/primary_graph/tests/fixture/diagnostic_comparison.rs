//! Real governed tree declarations for original-output inspection journeys.
use super::application_queries::{status_parameter, AccountSummaryParameters};
use super::{
    Account, AccountStatus, Activity, ActivitySequence, CapabilityDisclosure,
    CapabilityDisclosureBinding, IdentityExecutionSchema, TouchAccountCapability,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryInfluenceContract, ApplicationQueryLaneEligibility,
        ApplicationQueryMarkerIdentity, ApplicationQueryObservableInfluence,
        ApplicationQueryOrderingDirection, ApplicationQueryReference,
        ApplicationQueryResultShapeBuilder,
    },
    application_schema::{ApplicationEncodedScalarValue, ApplicationStructuredValueBinding},
};

#[path = "diagnostic_comparison/bindings.rs"]
mod bindings;
pub(in crate::domain_computation::primary_graph) use bindings::{
    DeniedDiagnosticFieldBinding, DeniedDiagnosticInternalBinding, DeniedDiagnosticRelationBinding,
    DiagnosticAccountBinding, GovernedDiagnosticAccountBinding,
};
#[path = "diagnostic_comparison/shape.rs"]
mod shape;
use shape::{activities, activity_identity, label, note, sequence, DiagnosticUnitBinding};
pub(in crate::domain_computation::primary_graph) use shape::{
    DiagnosticResult, DiagnosticResultBinding,
};

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
pub(super) enum DiagnosticPolicy {
    Complete,
    DenyInternal,
    DenyRelation,
    DenyField,
}

pub(super) fn definition<Q>(
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
        AccountStatus::reference(),
        value(),
        if policy == DiagnosticPolicy::DenyInternal {
            without_explanation()
        } else {
            all()
        },
    )
    .use_field_by(ActivitySequence::reference(), value(), all())
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
        .where_equal(AccountStatus::reference(), status_parameter())
        .order_by(sequence(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap()
}

pub(super) fn governed_metadata(
) -> worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition<
    IdentityExecutionSchema,
    GovernedDiagnosticAccountBinding,
> {
    use worth_query_declaration::facade::application_query::{
        ApplicationQueryInspectionDefinition, ApplicationQueryInspectionMetadataFacet,
        ApplicationQueryInspectionMetadataRef,
    };
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
