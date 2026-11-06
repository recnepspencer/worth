//! A demand can change discovery without changing its canonical root source input.
use super::super::hierarchy::{Child, ChildInput, HierarchyProgram, Input, Output, Parent};
use super::super::program::{Entry, EntryOutput};
use super::*;
pub(super) mod observation;
#[derive(Clone)]
pub struct Demand {
    pub source: String,
    pub discovery: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = first::Family;
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read::from(self.source.clone())
    }
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "recording-intent.non-leaf.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Parent> for Start {}
pub type RootConnection =
    ApplicationConnectionRef<Schema, Entry, EntryOutput, Parent, Input, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = first::initial::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &first::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: input.0.key.clone(),
            discovery: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(input.0.key == "b")),
        })
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "recording-intent.non-leaf.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type ChildConnection =
    ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        let key = if root.discovery.load(std::sync::atomic::Ordering::Relaxed) {
            "b"
        } else {
            "a"
        };
        Ok(query::first_output::Read(format!("first:{key}")))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        observation::record(&row.key);
        Ok(vec![second::Demand(
            row.key
                .strip_prefix("first:")
                .expect("real first output")
                .into(),
        )])
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    RootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("recording-intent.non-leaf");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        HierarchyProgram::feature_specs()
    }
}
