//! A born output consumes authored parent records before it owns any roster.
use super::super::schema::{Model as SourceEntity, ModelKey as SourceKey};
use super::*;
#[macro_use]
mod handler;
pub mod chains;
#[cfg(test)]
mod cold_contract;
pub mod composed;
#[cfg(test)]
mod courts;
pub mod program;
#[cfg(test)]
mod recapture;
#[cfg(test)]
mod retention;
pub mod supplier;
#[cfg(test)]
mod unowned_consumer;
#[cfg(test)]
mod wave_order;

type Scope = ApplicationMutationFieldScope<ModelKey>;
fn source_key() -> ModelKey {
    ModelKey::at("ModelKey")
}
fn source_value() -> ModelValue {
    ModelValue::at("ModelValue")
}
const PROFILE: &str = "captured-input";
pub fn output_key(row: &query::Row) -> String {
    format!("captured:{}", row.key)
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::model::Binding;
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
    fn source_intent(&self) -> query::model::Read {
        query::model::Read(self.0.clone())
    }
}

pub struct Capture;
impl WorthQueryProducerInputCaptureDefinition<Schema> for Capture {
    type OutputFamily = Family;
    type Parents = (program::SeedConnection, program::AnchorConnection);
    const IDENTITY: &'static str = "owned-outputs.captured-inputs.v1";
}

// These recipes select only this fixture's two lifecycle operations.
macro_rules! producer_capture_contract {
    () => {
        Some(WorthQueryProducerInputCaptureContract::new::<Capture>())
    };
}
macro_rules! producer_program_required {
    () => {
        true
    };
}
macro_rules! producer_input_reuse_contract {
    () => {
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::SCOPE,
        ))
    };
}
macro_rules! producer_operation {
    ($operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [Model, Member, FirstOutput];
            read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue"), FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
            read_relations: [ModelMember, UpstreamOf, FirstOf];
            writes: [FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
            creates: [FirstOutput]; links: [FirstOf];
        });
    };
}
phase!(
    initial,
    SourceScopeCascade,
    output,
    model,
    CapturedInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.captured-input.initial",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
phase!(
    preserve,
    SourceScopeCascade,
    output,
    model,
    CapturedPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.captured-input.preserve",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    preserve::declare(initial::declare(schema))
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.feature::<Roles>()?;
    Ok(())
}
pub type Roles = WorthQueryRequiredFeaturePair<
    Schema,
    program::Feature,
    initial::Producer,
    initial::Readiness,
    preserve::Producer,
    preserve::Readiness,
>;
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.producer_with_input_capture::<initial::Producer, Capture, _>(
        initial::Provider,
        initial::Handler {
            derive: |value| value,
        },
    )?;
    setup.producer_with_input_capture::<preserve::Producer, Capture, _>(
        preserve::Provider,
        preserve::Handler {
            derive: |value| value,
        },
    )?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}
