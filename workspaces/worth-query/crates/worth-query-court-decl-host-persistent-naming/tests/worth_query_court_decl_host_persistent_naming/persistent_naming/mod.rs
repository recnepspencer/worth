//! Persistent names through an installed Query world: scheme installation,
//! named seeds and tracked operation reads.
use worth_query_decl::facade::{
 application_operation::*, application_program::*, application_query::*,
    application_schema::*, authentication::*, *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};
mod schema;
use schema::*;
pub(crate) use schema::{GroupRole, Loop, Member, MemberRole, MEMBER_SCHEME};
mod branches;
mod cell_complex;
mod completeness;
pub(crate) mod coupled;
mod delete;
mod durable;
mod edit;
mod ergonomics;
pub(crate) mod groups;
pub(crate) mod invariant_names;
mod lineage;
mod named_read;
pub(crate) mod outputs;
pub(crate) mod probe;
mod query;
mod refusals;
mod retarget;
mod scope;
mod scoped;
mod stored;
mod unowned;
pub(crate) mod world;

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
    type Outputs = ApplicationProgramOutputs<coupled::producer::Graph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-model.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        // `stir` is an ordinary edit here: only the successor-round
        // program delivers it to its producers.
        let entry = entry()
            .action::<coupled::producer::successor::Binding, Completion>()
            .finish_action()
            .finish();
        vec![entry, coupled::producer::feature()]
    }
}
/// The entry feature's actions, shared with the successor-round program.
pub(crate) fn entry() -> ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, Entry> {
    ApplicationFeatureSpec::root::<Schema, Entry>()
        .provides::<coupled::producer::EntryOutput>()
        .action::<edit::model::Binding, Completion>()
        .finish_action()
        .action::<edit::member::Binding, Completion>()
        .finish_action()
        .action::<delete::Binding, Completion>()
        .finish_action()
        .action::<probe::Binding, Completion>()
        .finish_action()
        .action::<refusals::Binding, Completion>()
        .finish_action()
        .action::<outputs::sculpt::Binding, Completion>()
        .finish_action()
        .action::<stored::Binding, Completion>()
        .finish_action()
        .action::<retarget::Binding, Completion>()
        .finish_action()
        .action::<groups::source_currentness::Binding, Completion>()
        .finish_action()
        .action::<coupled::producer::shape::Binding, Completion>()
        .finish_action()
}
