use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use worth_query_consumer_values::{
    PlanarCurrentOutputExpectation, PlanarDerivedOutput, PlanarMutationDenial, PlanarOperation,
};
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryDecisionContextDependencies, WorthQueryProducerApplicability,
    WorthQueryProducerDemandResources, WorthQueryProducerInputReuseContract,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
};

use super::{PlanarMutationBinding, TopologySchemaBinding};

#[cfg(test)]
static PROVIDER_CONTACTS: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
pub(super) fn reset_provider_contacts() {
    PROVIDER_CONTACTS.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(super) fn provider_contacts() -> usize {
    PROVIDER_CONTACTS.load(Ordering::SeqCst)
}

const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Initial);
const PRESERVE: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Preserve);
const PRIMARY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];
const SUPPORTED: &[WorthQueryProducerApplicability] = &[
    INITIAL,
    PRESERVE,
    super::alternate_output::ALTERNATE_OUTPUT_APPLICABILITY,
];

pub struct PlanarOutputFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for PlanarOutputFamily {
    type Source = super::PlanarReadBinding<Schema>;
    type Entity = super::Body;

    const IDENTITY: &'static str = "worth.query.certification.planar-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = SUPPORTED;

    /// A body keyed `manual-` is certified by hand: its kind is served by
    /// the alternate producer alone, which declares no Preserve posture.
    fn profile_kind(source: &super::PlanarReadResult) -> &'static str {
        if source.body_key.starts_with("manual-") {
            "manual-certification"
        } else {
            "planar"
        }
    }
}

pub struct InitialPlanarProvider {
    authorization_denials: Arc<AtomicUsize>,
    domain_denial: Arc<AtomicBool>,
}

impl InitialPlanarProvider {
    pub fn new(authorization_denials: Arc<AtomicUsize>, domain_denial: Arc<AtomicBool>) -> Self {
        Self {
            authorization_denials,
            domain_denial,
        }
    }
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, InitialPlanarProducer<Schema>>
    for InitialPlanarProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.planar-initial-provider.v1";

    fn operation_input(&self, source: &super::PlanarReadResult) -> super::PlanarMutation {
        #[cfg(test)]
        PROVIDER_CONTACTS.fetch_add(1, Ordering::SeqCst);
        let mut input = planar_producer_input(source);
        if self
            .authorization_denials
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
        {
            input.scope_key.push_str(":authorization-denied");
        }
        if self.domain_denial.load(Ordering::SeqCst) {
            input.operation =
                PlanarOperation::VerifyCurrentOutputs(vec![PlanarCurrentOutputExpectation {
                    producer_key: source.body_key.clone(),
                    output_key: source.body_key.clone(),
                }]);
        }
        input
    }

    fn domain_denial_reason(&self, denial: &PlanarMutationDenial) -> Option<&'static str> {
        match denial {
            PlanarMutationDenial::CurrentOutputMissing => {
                Some("A current planar output is required before this decision.")
            }
            _ => None,
        }
    }

    fn idempotency_key(&self, _: &super::PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        #[cfg(test)]
        PROVIDER_CONTACTS.fetch_add(1, Ordering::SeqCst);
        planar_source_key(source_identity)
    }

    fn demand_resources(&self, _: &super::PlanarReadResult) -> WorthQueryProducerDemandResources {
        #[cfg(test)]
        PROVIDER_CONTACTS.fetch_add(1, Ordering::SeqCst);
        planar_producer_resources()
    }
}

pub struct InitialPlanarProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for InitialPlanarProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = InitialPlanarProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-initial.v1";
    type OutputRole = super::PlanarAnchorOutput<Schema>;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = PRIMARY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}

pub fn planar_producer_input(source: &super::PlanarReadResult) -> super::PlanarMutation {
    super::PlanarMutation {
        scope_key: source.body_key.clone(),
        operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
            body_key: source.body_key.clone(),
            value: worth_query_consumer_values::PositiveLength::new(
                worth_query_consumer_values::PositiveLength::get(&source.y) + 1,
            )
            .expect("a positive planar source has a positive successor"),
        }),
        validator_work: 4_096,
    }
}

pub fn planar_source_key(source_identity: &[u8; 32]) -> u64 {
    source_identity
        .chunks_exact(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("eight-byte identity chunk")))
        .fold(0, u64::wrapping_add)
}

pub const fn planar_producer_resources() -> WorthQueryProducerDemandResources {
    WorthQueryProducerDemandResources::new(4_096, 8_192)
}
