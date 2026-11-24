//! A later producer that writes the member `ring-drop` read and, when the
//! court asks, links `Beside` from member 0 of `ring-drop(a)`'s ring to a
//! record of its own, and on from that record to another ring record (#277
//! step 4): relations on that ring's records that the ring's own chain did
//! not make. Native refuses a relation between two outputs of one producer
//! that the producer did not make, so every link has this producer's record
//! at one end. Native also refuses a record of the ring's kind outside a
//! closed orbit, so that record is a ring of one.
use super::super::super::schema::{
    Model as SourceEntity, ModelKey as SourceKey, ModelValue as SourceValue,
};
use super::*;
type Scope = ApplicationMutationFieldScope<SourceKey>;
fn source_key() -> SourceKey {
    SourceKey::at("ModelKey")
}
fn source_value() -> SourceValue {
    SourceValue::at("ModelValue")
}
const PROFILE: &str = "ring-peer";
/// What one edit's round links. A court links in one edit only.
#[derive(Clone, Copy)]
pub enum Peer {
    /// Nothing.
    Idle,
    /// `Beside` from `ring:a:0` to this producer's own record.
    Own,
    /// The same, and `Beside` on from that record to this `IncidenceKey`.
    Onto(&'static str),
}
thread_local! {
    /// Per court, on the court's own thread.
    pub static LINK: std::cell::Cell<Peer> = const { std::cell::Cell::new(Peer::Idle) };
}
/// The `IncidenceKey` of the one record this producer owns.
pub const OWN: &str = "beside:a";
/// That record, as a member of this producer's outputs.
pub struct Own<C>(std::marker::PhantomData<fn() -> C>);
impl<C: ApplicationMutationOutputContract<Schema>> WorthQueryApplicationOutputRoleFamily
    for Own<C>
{
    type Schema = Schema;
    type Contract = C;
    type Entity = Incidence;
    const PREFIX: &'static str = "beside.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 0;
}
type Target = WorthQueryInvariantMutationTarget<Schema, Incidence>;
/// One decision: this producer's record if it exists, and the ends it links.
pub struct Capture {
    own: Option<Target>,
    from: Option<Target>,
    onto: Option<Target>,
}
pub fn output_key(row: &query::Row) -> String {
    format!("ring-peer:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value + 1000
}
pub fn capture<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
) -> Result<Capture, HandlerExecutionDenial>
where
    Incidence: worth_query_host::facade::domain::OperationReads<B::Operation>,
{
    let mut read = |name: &str| -> Result<Option<Target>, HandlerExecutionDenial> {
        let Some(node) =
            reader.resolve_optional_entity(IncidenceKey::at("IncidenceKey"), name.to_owned())?
        else {
            return Ok(None);
        };
        reader.entity(&node, Incidence::reference())?;
        Ok(Some(reader.mutation_target(&node)?))
    };
    let own = read(OWN)?;
    let (from, onto) = match LINK.get() {
        Peer::Idle => (None, None),
        Peer::Own => (read("ring:a:0")?, None),
        Peer::Onto(key) => (read("ring:a:0")?, read(key)?),
    };
    Ok(Capture { own, from, onto })
}
/// Keeps this producer's record, or creates it for its first link.
pub fn publish<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    source: &WorthQueryApplicationEffectEntity<Schema, SourceEntity>,
    peer: &Capture,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    Incidence: worth_query_host::facade::domain::OperationCreates<B::Operation>,
    Beside: worth_query_host::facade::domain::OperationLinks<B::Operation>,
    OwnedNext: worth_query_host::facade::domain::OperationLinks<B::Operation>,
{
    let own = match (&peer.own, &peer.from) {
        (Some(own), _) => {
            let own = writer.projected_entity(own)?;
            writer.preserve_member::<Own<B::Output>>(OWN, &own)?;
            own
        }
        (None, Some(_)) => {
            let own = writer.create_entity_in_context(
                source,
                Incidence::reference(),
                WorthQueryApplicationEntityKey::new(OWN).unwrap(),
            )?;
            writer.initialize_field(&own, IncidenceKey::at("IncidenceKey"), OWN.to_owned())?;
            writer.link(
                OwnedNext::reference(),
                "beside:a:ring".to_owned(),
                &own,
                &own,
            )?;
            writer.create_member::<Own<B::Output>>(OWN, &own)?;
            own
        }
        (None, None) => return Ok(()),
    };
    if let Some(from) = &peer.from {
        let from = writer.projected_entity(from)?;
        writer.link(Beside::reference(), "beside:a:in".to_owned(), &from, &own)?;
    }
    if let Some(onto) = &peer.onto {
        let onto = writer.projected_entity(onto)?;
        writer.link(Beside::reference(), "beside:a:out".to_owned(), &own, &onto)?;
    }
    Ok(())
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::producer_model::Binding;
    type Entity = SecondOutput;
    const IDENTITY: &'static str = PROFILE;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Initial),
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Preserve),
    ];
    fn profile_kind(_: &query::Row) -> &'static str {
        PROFILE
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read::from(self.0.clone())
    }
}
phase!(
    initial,
    SourceScopeCascade,
    output,
    producer_model,
    RingPeerInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.ring-peer.initial",
    SecondOutput,
    SecondFacts,
    SecondKey,
    SecondValue,
    SecondOf
);
phase!(
    preserve,
    SourceScopeCascade,
    output,
    producer_model,
    RingPeerPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.ring-peer.preserve",
    SecondOutput,
    SecondFacts,
    SecondKey,
    SecondValue,
    SecondOf
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    preserve::declare(initial::declare(schema))
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<initial::Producer>()?;
    contracts.producer::<preserve::Producer>()?;
    contracts.conditional::<initial::Readiness>()?;
    contracts.conditional::<preserve::Readiness>()?;
    Ok(())
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler { derive: derived })?;
    setup.handler::<preserve::Binding, _>(preserve::Handler { derive: derived })?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}

pub struct Feature;
pub struct InputPort;
pub struct OutputPort;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<InputPort, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "owned-outputs.ring-peer.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ring-peer-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ring-peer-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
