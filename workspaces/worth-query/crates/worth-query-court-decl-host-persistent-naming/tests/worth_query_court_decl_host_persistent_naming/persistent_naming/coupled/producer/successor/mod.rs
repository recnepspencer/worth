//! A coupled producer's successor round (PR C §1.5, #277) reading names.
//!
//! A `stir` edit of model `b` runs the `stir-root` producer and, discovered
//! from its output, the `stir-child` producer. The root's Preserve reads
//! member `reference`'s value and, when a drop is registered for the edit's
//! value, reads and drops one named member and one named seam. The child
//! then writes `reference`, so the root reopens as a successor round of its
//! own chain: it reads the member and seam its own earlier round dropped as
//! they were at the chain head, where they are live, and drops them again.
//!
//! The two producers live only in their own program (`world`): the shared
//! persistent-naming program never installs them.
use super::*;

pub mod child;
mod courts;
pub mod root;
mod world;
pub use world::{installing, SuccessorWorld};

/// The authored edit: writes model `key`'s value; the producers follow.
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.stir.v1" });
worth_query_operation!(pub Stir for Schema, input Input; access {
    read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue")];
    writes: [ModelValue::at("ModelValue")];
});
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.stir.v1", input Input, operation Stir, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.stir-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.stir-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, edit::Denial> {
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), input.key.clone())?;
            reader.field(&model, ModelValue::at("ModelValue"))?;
            reader.mutation_target(&model)
        })();
        match result {
            Ok(model) => HandlerResult::Completed(model),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Intent,
        model: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = writer.projected_entity(&model).and_then(|model| {
            writer.write_field(&model, ModelValue::at("ModelValue"), input.value)
        });
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub struct StirStart;
impl ApplicationConnectionIdentity for StirStart {
    const IDENTITY: &'static str = "persistent-naming.stir-start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, root::Feature> for StirStart {}
pub type StartConnection =
    ApplicationConnectionRef<Schema, Entry, EntryOutput, root::Feature, root::InputPort, StirStart>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for StirStart {
    type Source = Binding;
    type Demand = root::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &Intent,
    ) -> Result<root::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(root::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, StirStart> for Binding {
    fn demand_from_source(
        input: &Intent,
    ) -> Result<root::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <StirStart as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}

pub struct StirDiscover;
impl ApplicationConnectionIdentity for StirDiscover {
    const IDENTITY: &'static str = "persistent-naming.stir-discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, root::Feature, child::Feature>
    for StirDiscover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    root::Feature,
    root::OutputPort,
    child::Feature,
    child::InputPort,
    StirDiscover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for StirDiscover {
    type RootDemand = root::Demand;
    type Discovery = query::member::Read;
    type Demand = child::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        demand: &root::Demand,
    ) -> Result<query::member::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::member::Read(root::output_key(&demand.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<child::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let model = row
            .key
            .strip_prefix("stir-root:")
            .expect("a root output key");
        Ok(vec![child::Demand(model.to_owned())])
    }
}

pub type Tree = ApplicationOutputGraph<
    StartConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "persistent-naming.stir-delivery";
}
impl ApplicationFixedActionDelivery<Schema, Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    child::declare(root::declare(schema))
        .declared_operation::<Stir>(
            Stir::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    root::contracts(contracts)?;
    child::contracts(contracts)
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    root::install(setup)?;
    child::install(setup)?;
    setup.handler::<Binding, _>(Handler)
}
