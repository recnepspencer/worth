//! Generic document ownership courts, independent of CAD declarations.
use worth_query_decl::facade::{
    application_history::*, application_operation::*, application_program::*, application_query::*,
    application_schema::*, authentication::*, *,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationHistoryDenial;
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};
mod schema;
use schema::*;
mod context;
mod edit;
mod exclusivity;
mod generation;
mod gesture;
mod ownership_change;
mod query;
mod recording;
mod world;

pub struct History;
worth_query_portable_type!(History => "worth.owned-model.history.v1");
impl ApplicationHistoryBinding<Schema> for History {
    type Access = query::model::Read;
    const IDENTITY: &'static str = "OwnedModelHistory";
}
pub struct ResultLimitedHistory;
worth_query_portable_type!(ResultLimitedHistory => "worth.owned-model.result-limited-history.v1");
impl ApplicationHistoryBinding<Schema> for ResultLimitedHistory {
    type Access = query::model::Read;
    const IDENTITY: &'static str = "ResultLimitedOwnedHistory";
    const GROUP_RESOURCES: ApplicationHistoryGroupResources = ApplicationHistoryGroupResources {
        maximum_members: std::num::NonZeroUsize::new(4).unwrap(),
        maximum_retained_results: std::num::NonZeroUsize::new(1).unwrap(),
    };
}
pub struct Entry;
impl ApplicationFeature<Schema> for Entry {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "owned-model.entry.v1";
}
pub struct Completion;
impl<B: ApplicationMutationBinding<Schema>> ApplicationActionDelivery<Schema, B> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "owned-model.completion.v1";
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-model.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Entry>()
            .action::<edit::model::Binding, Completion>()
            .finish_action()
            .action::<edit::member::Binding, Completion>()
            .finish_action()
            .action::<generation::Binding, Completion>()
            .finish_action()
            .action::<ownership_change::Binding, Completion>()
            .finish_action()
            .finish()]
    }
}
