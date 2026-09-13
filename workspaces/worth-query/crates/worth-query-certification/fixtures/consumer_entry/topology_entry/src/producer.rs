use std::marker::PhantomData;

use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerOutputFamily,
};
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;

use super::{PlanarMutationBinding, TopologySchemaBinding};

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "planar",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "planar",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const SUPPORTED: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];

pub struct PlanarOutputFamily;

impl WorthQueryProducerOutputFamily for PlanarOutputFamily {
    const IDENTITY: &'static str = "worth.query.certification.planar-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = SUPPORTED;
}

pub struct InitialPlanarProvider;

impl WorthQueryApplicationProducerProvider for InitialPlanarProvider {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.planar-initial-provider.v1";
}

pub struct PreservePlanarProvider;

impl WorthQueryApplicationProducerProvider for PreservePlanarProvider {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.planar-preserve-provider.v1";
}

pub struct InitialPlanarProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for InitialPlanarProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type Source = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = InitialPlanarProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-initial.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

pub struct PreservePlanarProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for PreservePlanarProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type Source = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = PreservePlanarProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-preserve.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[PRESERVE];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}
