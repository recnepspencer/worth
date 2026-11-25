//! A coupled producer that names its outputs. An edit of model `key`
//! (`shape`, which declares the `pieces` required output) runs the producer
//! in the same envelope: its Initial creates a primary and a secondary member
//! and a seam between them and names all three through one naming scope; its
//! Preserve re-derives them in place. Relational composes the producer's
//! naming plan with the edit's own into the envelope's one plan.
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[macro_use]
mod readiness;
#[macro_use]
mod binding;
mod courts;
mod group_courts;
pub mod group_reads;
pub mod initial;
pub mod preserve;
mod reading;
pub mod reads;
pub mod shape;
pub mod successor;

pub const PROFILE: &str = "pieces";

/// How the producer's next Initial for one model names its outputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Naming {
    #[default]
    Complete,
    /// Leaves the secondary member unnamed: the envelope must refuse.
    LeaveSecondaryUnnamed,
}

static NAMING: Mutex<BTreeMap<String, Naming>> = Mutex::new(BTreeMap::new());

pub fn name_outputs(model: &str, naming: Naming) {
    NAMING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(model.to_owned(), naming);
}

fn naming(model: &str) -> Naming {
    NAMING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(model)
        .copied()
        .unwrap_or_default()
}

pub fn primary_key(model: &str) -> String {
    format!("pieces-{model}-p")
}

pub fn secondary_key(model: &str) -> String {
    format!("pieces-{model}-s")
}

pub fn seam_key(model: &str) -> String {
    format!("pieces-{model}-seam")
}

/// The occurrence every output of one model's producer is named under.
pub fn occurrence(model: &str) -> String {
    format!("pieces-{model}")
}

pub struct Derived;
impl ApplicationFeature<Schema> for Derived {
    type Inputs = ApplicationFeatureInputList<PiecesInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "persistent-naming.derived.v1";
}
pub struct EntryOutput;
impl ApplicationOutputPort<Schema, Entry> for EntryOutput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "persistent-naming.entry-output.v1";
}
pub struct PiecesInput;
impl ApplicationInputPort<Schema, Derived> for PiecesInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "pieces-input";
    const REQUIRED: bool = false;
}
pub struct StartPieces;
impl ApplicationConnectionIdentity for StartPieces {
    const IDENTITY: &'static str = "persistent-naming.start-pieces";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Derived> for StartPieces {}
pub type PiecesConnection =
    ApplicationConnectionRef<Schema, Entry, EntryOutput, Derived, PiecesInput, StartPieces>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for StartPieces {
    type Source = initial::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &initial::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
pub type Graph = ApplicationDemandOutputGraph<PiecesConnection, ApplicationOutputLeaf>;

pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::model::Binding;
    type Entity = Member;
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
    fn source_intent(&self) -> query::model::Read {
        query::model::Read(self.0.clone())
    }
}

/// The derived feature whose conditional operations are the two phases.
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Derived>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    successor::declare(shape::declare(preserve::declare(initial::declare(schema))))
}

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<initial::Producer>()?;
    contracts.producer::<preserve::Producer>()?;
    contracts.conditional::<initial::Readiness>()?;
    contracts.conditional::<preserve::Readiness>()?;
    // Only the successor-round world installs its producers (`successor`).
    if successor::installing() {
        successor::contracts(contracts)?;
    }
    Ok(())
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler)?;
    setup.handler::<preserve::Binding, _>(preserve::Handler)?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())?;
    successor::install(setup)?;
    setup.handler::<shape::Binding, _>(shape::Handler)
}
