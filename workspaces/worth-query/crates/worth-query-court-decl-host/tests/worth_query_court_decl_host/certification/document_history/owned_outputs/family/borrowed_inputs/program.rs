//! A concrete collector recipe over the existing member and model producers.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::super::super::{hierarchy::program as hierarchy, program as entry};
use super::*;

pub struct Feature;
pub struct RootInput;
pub struct MemberInput;
pub struct HeaderInput;
pub struct Output;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<
        RootInput,
        ApplicationFeatureInputList<MemberInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "owned-outputs.borrowed-inputs.feature";
}
macro_rules! input {
    ($name:ident, $identity:literal) => {
        impl ApplicationInputPort<Schema, Feature> for $name {
            type Value = query::RowBinding;
            const IDENTITY: &'static str = $identity;
            const REQUIRED: bool = false;
        }
    };
}
input!(RootInput, "borrowed-inputs.root");
input!(MemberInput, "borrowed-inputs.member");
impl ApplicationOutputPort<Schema, Feature> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "borrowed-inputs.output";
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "borrowed-inputs.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, Feature> for Start {}
type Root =
    ApplicationConnectionRef<Schema, entry::Entry, entry::EntryOutput, Feature, RootInput, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = fixture::model::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::model::Binding {
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
macro_rules! model_root {
    ($binding:ty, $input:ty) => {
        impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for $binding {
            fn demand_from_source(
                _: &$input,
            ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(Demand("a".into()))
            }
        }
    };
}
model_root!(edit::Binding, edit::Intent);
model_root!(member_edit::Binding, member_edit::Intent);
model_root!(disconnect::Binding, disconnect::Intent);
model_root!(
    super::super::super::synchronous_edit::interleaved::created::Binding,
    super::super::super::synchronous_edit::interleaved::created::Intent
);
model_root!(
    authored_member::retire::Binding,
    authored_member::retire::Intent
);
model_root!(
    authored_member::restore::Binding,
    authored_member::restore::Intent
);

pub struct Members;
impl ApplicationConnectionIdentity for Members {
    const IDENTITY: &'static str = "borrowed-inputs.members";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Child, Feature> for Members {}
type MemberConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Child,
    hierarchy::ChildOutput,
    Feature,
    MemberInput,
    Members,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Members {
    type RootDemand = downstream::Demand;
    type Discovery = owners::Read;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &downstream::Demand,
    ) -> Result<owners::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owners::Read(root.0.clone()))
    }
    fn demands_from_discovery(
        row: &owners::Owners,
    ) -> Result<Vec<Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(row
            .owners
            .iter()
            .map(|row| Demand(row.key.clone()))
            .collect())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Members {
    type Prerequisites = query::members::Read;
    type PrerequisiteRow = query::RowBinding;
    type Selection = ApplicationQueryResultRelationRef<
        query::members::Query,
        query::members::MembersSlot,
        Schema,
        ModelMember,
        Model,
        Member,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        query::members::members()
    }
    fn prerequisites_from_target(
        target: &Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<downstream::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(downstream::Demand(row.key.clone()))
    }
}
pub struct Inputs;
impl ApplicationOccurrenceComponent<Schema> for Inputs {
    type Connections = (MemberConnection, HeaderConnection);
}
pub type Tree = ApplicationOutputGraph<Root, ApplicationOutputOccurrences<Inputs>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, (hierarchy::Tree, hierarchy::SeedTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.borrowed-inputs.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = entry::envelope_actions::<
            fixture::Delivery<Tree>,
            Completion,
            Completion,
            Completion,
            Completion,
            Completion,
            Completion,
        >(entry::entry_builder_with_retirement::<
            fixture::Delivery<Tree>,
            fixture::Delivery<Tree>,
            fixture::Delivery<Tree>,
            fixture::Delivery<Tree>,
        >())
        .action::<super::super::super::synchronous_edit::interleaved::created::Binding, Completion>(
        )
        .finish_action()
        .action::<member_retarget::operation::Binding, Completion>()
        .finish_action()
        .finish();
        specs
    }
}

impl ApplicationInputPort<Schema, hierarchy::Child> for HeaderInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "borrowed-inputs.header-parent";
    const REQUIRED: bool = false;
}
pub struct HeaderParent;
impl ApplicationConnectionIdentity for HeaderParent {
    const IDENTITY: &'static str = "borrowed-inputs.header-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Feature, hierarchy::Child> for HeaderParent {}
type HeaderConnection =
    ApplicationConnectionRef<Schema, Feature, Output, hierarchy::Child, HeaderInput, HeaderParent>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for HeaderParent {
    type RootDemand = Demand;
    type Demand = from_first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("borrowed:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![from_first::Demand(row.key.clone())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for HeaderParent {
    type Prerequisites = header_owners::Read;
    type PrerequisiteRow = query::RowBinding;
    type Selection = ApplicationQueryResultRelationRef<
        header_owners::Query,
        header_owners::OwnersSlot,
        Schema,
        FirstOf,
        FirstOutput,
        Model,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        header_owners::owners()
    }
    fn prerequisites_from_target(
        target: &from_first::Demand,
    ) -> Result<Self::Prerequisites, WorthQueryRequiredOutputConnectionDenial> {
        Ok(header_owners::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(row.key.clone()))
    }
}

pub(in super::super::super) fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<Output>()
        .conditional_mutation::<initial::Binding>()
        .conditional_mutation::<preserve::Binding>()
        .finish()
}
