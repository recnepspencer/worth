use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryLiveCauseBinding, ApplicationQueryLiveResourceContract,
    ApplicationQueryOrderingDirection, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ForwardResultTraversal,
    ManyResults,
};
use worth_query_host::facade::{declaration, primary_graph, worth_query_application_query};

use super::{
    IntentFacts, IntentIdentityField, IntentLiveTarget, TemporalExecutionEffect,
    TemporalExecutionNotice, TemporalHostSchema, TemporalIntent,
};

pub struct IntentLiveQueryParameters;
pub struct IntentLiveIdentitySlot;
pub struct IntentLiveTargetSlot;
pub struct IntentLiveTargetIdentitySlot;
worth_query_host::facade::worth_query_portable_type!(
    IntentLiveIdentitySlot => "worth.query.test.host.temporal.live_identity_slot.v1"
);
worth_query_host::facade::worth_query_portable_type!(
    IntentLiveTargetSlot => "worth.query.test.host.temporal.live_target_slot.v1"
);
worth_query_host::facade::worth_query_portable_type!(
    IntentLiveTargetIdentitySlot => "worth.query.test.host.temporal.live_target_identity_slot.v1"
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentLiveQueryResult {
    pub identity: String,
    pub targets: Vec<String>,
}
worth_query_host::facade::worth_query_portable_type!(
    IntentLiveQueryResult => "worth.query.test.host.temporal.live_result.v1"
);
worth_query_host::facade::worth_query_structured_value_binding!(pub IntentLiveQueryParametersBinding for IntentLiveQueryParameters { identity: "IntentLiveQueryParameters" });
worth_query_host::facade::worth_query_structured_value_binding!(pub IntentLiveQueryResultBinding for IntentLiveQueryResult { identity: "worth.query.test.host.temporal.live_result.v1" });
worth_query_host::facade::worth_query_structured_value_binding!(IntentLiveNestedResultBinding for () { identity: "worth.rust.unit" });

worth_query_application_query!(
    pub TemporalIntentLiveQuery for TemporalHostSchema,
    identity "TemporalIntentLiveQuery",
    parameters IntentLiveQueryParametersBinding,
    result IntentLiveQueryResultBinding,
    scope TemporalIntent => "TemporalIntent",
    name "temporal_intent_live_query"
);

pub fn temporal_intent_live_query_definition() -> ApplicationQueryDefinition<
    TemporalHostSchema,
    TemporalIntentLiveQuery,
    IntentLiveQueryParameters,
    IntentLiveQueryResult,
    TemporalIntent,
> {
    let target = ApplicationQueryResultShapeBuilder::<
        TemporalHostSchema,
        TemporalIntentLiveQuery,
        TemporalIntent,
        (),
        IntentLiveNestedResultBinding,
    >::new(TemporalIntent::reference())
    .field(target_identity());
    let shape = ApplicationQueryResultShapeBuilder::<
        TemporalHostSchema,
        TemporalIntentLiveQuery,
        TemporalIntent,
        IntentLiveQueryResult,
        IntentLiveQueryResultBinding,
    >::new(TemporalIntent::reference())
    .field(scope_identity())
    .relation(live_targets(), target)
    .build();
    ApplicationQueryDefinitionBuilder::declare(TemporalIntentLiveQuery::reference())
        .root(TemporalIntent::reference())
        .scope(TemporalIntent::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .order_by(
            target_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(live_targets())
        .live_by::<TemporalIntent, TemporalIntentLiveCause, _, _, _, _, _, _, _, _>(
            scope_identity(),
            target_identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .expect("the temporal live query is canonical")
}

impl primary_graph::WorthQueryApplicationProjection<TemporalHostSchema, TemporalIntentLiveQuery>
    for IntentLiveQueryResult
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            TemporalHostSchema,
            TemporalIntentLiveQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        let targets = row
            .many(live_targets())?
            .iter()
            .map(|target| target.field(target_identity()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            identity: row.field(scope_identity())?,
            targets,
        })
    }
}

pub struct TemporalIntentLiveCause;
worth_query_host::facade::worth_query_portable_type!(
    TemporalIntentLiveCause => "worth.query.test.host.temporal.intent_live_cause.v1"
);

impl
    ApplicationQueryLiveCauseBinding<
        TemporalHostSchema,
        TemporalIntentLiveQuery,
        TemporalIntent,
        TemporalIntent,
    > for TemporalIntentLiveCause
{
    type Effect = TemporalExecutionEffect;
    type PayloadBinding = super::TemporalExecutionNoticeBinding;
    type ScopeIdentityBinding = declaration::application_schema::StringApplicationValueBinding;
    type TargetIdentityBinding = declaration::application_schema::StringApplicationValueBinding;

    fn effect() -> declaration::application_schema::ApplicationEffectRef<
        TemporalHostSchema,
        Self::Effect,
        TemporalExecutionNotice,
    > {
        TemporalExecutionEffect::reference()
    }

    fn scope_identity(payload: &TemporalExecutionNotice) -> String {
        payload.identity.clone()
    }

    fn target_identity(payload: &TemporalExecutionNotice) -> String {
        payload.identity.clone()
    }
}

type IdentityResultField<Slot> = ApplicationQueryResultFieldRef<
    TemporalIntentLiveQuery,
    Slot,
    TemporalHostSchema,
    TemporalIntent,
    IntentFacts,
    IntentIdentityField,
    String,
    declaration::application_schema::ReadOnly,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
>;

fn scope_identity() -> IdentityResultField<IntentLiveIdentitySlot> {
    ApplicationQueryResultFieldRef::new("identity", IntentIdentityField::reference())
}

fn target_identity() -> IdentityResultField<IntentLiveTargetIdentitySlot> {
    ApplicationQueryResultFieldRef::new("target_identity", IntentIdentityField::reference())
}

fn live_targets() -> ApplicationQueryResultRelationRef<
    TemporalIntentLiveQuery,
    IntentLiveTargetSlot,
    TemporalHostSchema,
    IntentLiveTarget,
    TemporalIntent,
    TemporalIntent,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("live_targets", IntentLiveTarget::reference())
}
