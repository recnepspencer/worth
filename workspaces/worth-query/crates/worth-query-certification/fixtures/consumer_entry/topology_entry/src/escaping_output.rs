//! A genuine external-contract producer, kept in its own family for eligibility courts.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::application_contribution::*;

mod binding;
pub use binding::*;
mod connection;
pub use connection::*;
mod readiness;
pub use readiness::EscapingPlanarReadiness;

pub struct EscapingPlanarFamily;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "escaping-planar",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl<S: TopologySchemaBinding> WorthQueryProducerOutputFamily<S> for EscapingPlanarFamily {
    type Source = PlanarReadBinding<S>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.escaping-planar-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "escaping-planar"
    }
}
pub struct EscapingPlanarProducer<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> WorthQueryApplicationProducerBinding<S>
    for EscapingPlanarProducer<S>
{
    type Operation = EscapingPlanarBinding<S>;
    type OutputFamily = EscapingPlanarFamily;
    type Provider = EscapingPlanarProvider;
    type OutputRole = PlanarAnchorOutput<S>;
    const IDENTITY: &'static str = "worth.query.certification.escaping-planar-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const REUSE_POLICY: &'static str = "exact-source";
}
pub struct EscapingPlanarProvider;
impl<S: TopologySchemaBinding> WorthQueryApplicationProducerProvider<S, EscapingPlanarProducer<S>>
    for EscapingPlanarProvider
{
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.escaping-planar-provider.v1";
    fn operation_input(&self, source: &PlanarReadResult) -> EscapingPlanarInput {
        EscapingPlanarInput(planar_producer_input(source))
    }
    fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        planar_source_key(identity)
    }
}

#[derive(Clone)]
pub struct EscapingPlanarDemand(pub PlanarRead);
impl<S: TopologySchemaBinding> WorthQueryApplicationOutputDemand<S> for EscapingPlanarDemand {
    type OutputFamily = EscapingPlanarFamily;
    fn source_intent(&self) -> PlanarRead {
        self.0.clone()
    }
}

pub(crate) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = EscapePlanar::reference::<S>();
    let effect = EscapingPlanarNotice::reference::<S>();
    schema
        .effect(effect)
        .operation(
            operation
                .definition()
                .external_effect(
                    effect,
                    WorthQueryExternalEffectCorrelationFamily::new("escaping-planar-notice")
                        .unwrap(),
                )
                .no_aftermath()
                .finish(),
        )
        .operation_emit(operation, effect)
        .operation_projection_work_budget(operation, 256)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionX::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_read_field(operation, Length::reference())
        .operation_read_relation(operation, PlanarSuccessor::reference())
        .operation_create(operation, Body::reference())
        .operation_write(operation, BodyKey::reference())
        .operation_write(operation, PositionX::reference())
        .operation_write(operation, PositionY::reference())
        .operation_write(operation, Length::reference())
        .operation_link(operation, PlanarSuccessor::reference())
        .operation_unlink(operation, PlanarSuccessor::reference())
        .application_mutation_binding::<EscapingPlanarBinding<S>>()
}
