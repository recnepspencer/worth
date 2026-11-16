//! A model-owned header borrows its members' actual produced values, owning none of them.
use super::super::schema::{
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
const PROFILE: &str = "borrowed-inputs";
pub fn output_key(row: &query::Row) -> String {
    format!("borrowed:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value
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
macro_rules! producer_program_required {
    () => {
        true
    };
}
macro_rules! producer_operation {
    ($operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [Model, Member, ChildOutput, FirstOutput];
            read_fields: [source_key(), source_value(), FirstKey::at("FirstKey"), FirstValue::at("FirstValue"), MemberKey::at("MemberKey"), ChildKey::at("ChildKey"), ChildValue::at("ChildValue")];
            read_relations: [ModelMember, FirstOf];
            creates: [FirstOutput]; writes: [FirstKey::at("FirstKey"), FirstValue::at("FirstValue")]; links: [FirstOf];
        });
    };
}
macro_rules! producer_decision_dependency {
    ($operation:ident, $reader:ident, $source:ident) => {{
        let mut sum = 0;
        for link in $reader
            .relations_from(ModelMember::reference(), &$source)
            .map_err(HandlerExecutionDenial::new)?
        {
            let key = $reader
                .field(link.to(), MemberKey::at("MemberKey"))?
                .expect("actual member key");
            let child =
                $reader.resolve_entity(ChildKey::at("ChildKey"), format!("downstream:{key}"))?;
            sum += $reader
                .field(&child, ChildValue::at("ChildValue"))?
                .expect("actual parent output");
        }
        sum
    }};
}
phase!(
    initial,
    SourceScopeCascade,
    output,
    producer_model,
    BorrowedInputsInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.borrowed-inputs.initial",
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
    producer_model,
    BorrowedInputsPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.borrowed-inputs.preserve",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    disconnect::declare(header_owners::declare(owners::declare(preserve::declare(
        initial::declare(schema),
    ))))
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
    setup.handler::<disconnect::Binding, _>(disconnect::Handler)?;
    setup.handler::<initial::Binding, _>(initial::Handler { derive: derived })?;
    setup.handler::<preserve::Binding, _>(preserve::Handler { derive: derived })?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}

#[cfg(test)]
mod courts;
pub(in super::super) mod disconnect;
mod header_owners;
mod owners;
pub mod program;
