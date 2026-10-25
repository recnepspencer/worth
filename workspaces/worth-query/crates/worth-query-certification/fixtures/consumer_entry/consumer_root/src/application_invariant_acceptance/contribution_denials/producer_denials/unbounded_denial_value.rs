//! A producer whose declared denial value has no finite wire bound is refused
//! at installation, before initial state, never at its first denial.
use super::*;
use worth_query_consumer_values::PlanarMutationDenial;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryProducerDenialProjection, WorthQueryProjectedProducerDenial,
    },
    domain::InternedString,
};

producer_schema!(UnboundedDenialSchema, UnboundedDenialContribution);

pub(super) fn unbounded_denial_value_is_denied_before_initial_state() {
    let calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory_program(
        validated_denial_program::<UnboundedDenialSchema>(),
        UnboundedDenialSchema::declaration().unwrap(),
        (Arc::clone(&calls), Arc::clone(&calls)),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits(),
        |_, _, _| panic!("an unbounded denial value must deny before initial state"),
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ProducerDenialValueUnbounded,
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

worth_query_decl::facade::worth_query_value_binding!(UnboundedText for String {
    identity: "worth.query.certification.unbounded-denial-text.v1", scalar: String,
    validate: any_text, encode: text, decode: from_text,
});
fn any_text(_: &String) -> Result<(), &'static str> {
    Ok(())
}
fn text(value: &str) -> InternedString {
    value.to_owned().into()
}
fn from_text(value: InternedString) -> Option<String> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    Some(value)
}

struct NoEvidence;
impl WorthQueryProducerDenialProjection<PlanarMutationDenial> for NoEvidence {
    type Binding = UnboundedText;
    fn project(_: &PlanarMutationDenial) -> Option<String> {
        None
    }
}

struct UnboundedDenialProvider;
impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, UnboundedDenialProducer<Schema>>
    for UnboundedDenialProvider
{
    type DenialValue = WorthQueryProjectedProducerDenial<NoEvidence>;
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.unbounded-denial.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> PlanarMutation {
        worth_query_topology_entry::planar_producer_input(source)
    }

    fn idempotency_key(&self, _: &PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        worth_query_topology_entry::planar_source_key(source_identity)
    }
}

/// The initial planar producer's contract, provided with an unbounded value.
struct UnboundedDenialProducer<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for UnboundedDenialProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = UnboundedDenialProvider;

    const IDENTITY: &'static str = "worth.query.certification.unbounded-denial-producer.v1";
    type OutputRole = PlanarAnchorOutput<Schema>;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Initial),
        WorthQueryProducerApplicability::new(
            "planar",
            WorthQueryProducerLifecyclePosture::Preserve,
        ),
    ];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const REUSE_POLICY: &'static str = "exact-source";
}

struct UnboundedDenialContribution;
impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for UnboundedDenialContribution
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;

    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::register_members(builder)
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for UnboundedDenialContribution
{
    type Configuration = Arc<AtomicUsize>;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts
            .producer::<UnboundedDenialProducer<Schema>>()?
            .producer::<worth_query_topology_entry::AlternatePlanarOutputProducer<Schema>>()?
            .producer::<worth_query_topology_entry::PlanarFinalOutputProducer<Schema>>()?
            .producer::<worth_query_topology_entry::PlanarFinalPreserveProducer<Schema>>()?;
        Ok(())
    }

    fn configure(
        calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        install_topology_behavior(setup)?;
        setup.producer::<UnboundedDenialProducer<Schema>>(UnboundedDenialProvider)
    }
}
