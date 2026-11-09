//! A real installed root whose caller-owned demand may drift after retention.
use super::*;
#[derive(Clone)]
pub struct Demand(pub std::sync::Arc<std::sync::atomic::AtomicBool>);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = first::Family;
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read::from(
            if self.0.load(std::sync::atomic::Ordering::Relaxed) {
                "b"
            } else {
                "a"
            }
            .to_owned(),
        )
    }
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.recording-intent.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::Entry, program::Derived> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    program::Entry,
    program::EntryOutput,
    program::Derived,
    program::FirstInput,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = first::initial::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &first::initial::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        )))
    }
}
pub type Tree = ApplicationDemandOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.recording-intent.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::Program::feature_specs()
    }
}
