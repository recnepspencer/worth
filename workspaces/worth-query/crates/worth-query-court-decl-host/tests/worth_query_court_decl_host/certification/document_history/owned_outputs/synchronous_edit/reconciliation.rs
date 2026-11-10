//! A generic fan-out whose shrink re-spaces survivors and clears one-round facts.
use super::super::*;
use family::{readiness_query, Denial, DenialBinding, ResultBinding};
use std::marker::PhantomData;
use worth_foundational::facade::ProducerPublicationOwnership;
#[macro_use]
mod phase;
#[macro_use]
mod child_phase;
mod cap_edit;
mod child;
mod comparison;
mod courts;
mod demand_after_shrink;
mod discovery;
mod emission;
mod observation;
pub(crate) mod peer_caps;
mod program;
pub use program::{child_feature, feature};
mod publication;
mod snapshot;

type SourceEntity = Model;
type SourceKey = ModelKey;
type Scope = ApplicationMutationFieldScope<ModelKey>;
const PROFILE: &str = "reconciliation";
fn source_key() -> ModelKey {
    ModelKey::at("ModelKey")
}
fn source_value() -> ModelValue {
    ModelValue::at("ModelValue")
}
fn output_key(row: &query::Row) -> String {
    format!("reconciled:root:{}", row.key)
}

pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::producer_model::Binding;
    type Entity = FirstOutput;
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
trait MemberKind: ApplicationEntityMarkerIdentity<Schema> {
    const PREFIX: &'static str;
}
impl MemberKind for FirstOutput {
    const PREFIX: &'static str = "member.";
}
impl MemberKind for SecondOutput {
    const PREFIX: &'static str = "source.";
}
impl MemberKind for ChildOutput {
    const PREFIX: &'static str = "retired.";
}
pub struct Rows<C, E>(PhantomData<fn() -> (C, E)>);
impl<C: ApplicationMutationOutputContract<Schema>, E: MemberKind + 'static>
    WorthQueryApplicationOutputRoleFamily for Rows<C, E>
{
    type Schema = Schema;
    type Contract = C;
    type Entity = E;
    const PREFIX: &'static str = E::PREFIX;
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 0;
}
reconciliation_phase!(
    initial,
    SourceScopeCascade,
    producer_model,
    ReconciledInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.reconciliation.initial",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
reconciliation_phase!(
    preserve,
    SourceScopeCascade,
    producer_model,
    ReconciledPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.reconciliation.preserve",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    cap_edit::declare(discovery::declare(snapshot::declare(child::declare(
        preserve::declare(initial::declare(
            schema.effect(emission::Changed::reference()),
        )),
    ))))
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.feature::<Roles>()?;
    child::contracts(contracts)?;
    Ok(())
}
pub type Roles = WorthQueryRequiredFeaturePair<
    Schema,
    program::Root,
    initial::Producer,
    initial::Readiness,
    preserve::Producer,
    preserve::Readiness,
>;
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    indices: bool,
    peer_caps: bool,
    emission: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    child::install(setup, indices)?;
    setup.handler::<cap_edit::Binding, _>(cap_edit::Handler)?;
    setup.handler::<initial::Binding, _>(initial::Handler {
        derive: |v| v,
        peer_caps,
        emission,
    })?;
    setup.handler::<preserve::Binding, _>(preserve::Handler {
        derive: |v| v,
        peer_caps,
        emission,
    })?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}

mod adoption;

mod cold;

mod native_unchanged;

mod appendix_effect;

mod commit_count;

mod upgrade;

mod changed_source;

mod unkeyed;

mod kept_refusals;
mod late_stop;
mod original_root;
