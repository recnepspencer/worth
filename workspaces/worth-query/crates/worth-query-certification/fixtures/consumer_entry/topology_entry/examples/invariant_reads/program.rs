//! The reference application with an ordinary, declared topology-edit action.
pub use super::reference_program::{ExampleConfiguration, ExampleSchema, Root};
use super::*;

pub struct ExampleProgram<const ERASE: bool>;
impl<const ERASE: bool> ApplicationProgramDefinition<ExampleSchema> for ExampleProgram<ERASE> {
    type Contributions = <reference_program::ExampleProgram<ERASE> as ApplicationProgramDefinition<
        ExampleSchema,
    >>::Contributions;
    type Outputs = <reference_program::ExampleProgram<ERASE> as ApplicationProgramDefinition<
        ExampleSchema,
    >>::Outputs;
    type Rules = <reference_program::ExampleProgram<ERASE> as ApplicationProgramDefinition<
        ExampleSchema,
    >>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("invariant-read-example");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = reference_program::ExampleProgram::<ERASE>::feature_specs();
        features.push(
            ApplicationFeatureSpec::root::<ExampleSchema, TopologyEdits>()
                .action::<PlanarEditBinding<ExampleSchema>, EditCompletion>()
                .finish_action()
                .finish(),
        );
        features
    }
}

struct TopologyEdits;
impl ApplicationFeature<ExampleSchema> for TopologyEdits {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "invariant-read-topology-edits";
}
struct EditCompletion;
impl ApplicationActionDelivery<ExampleSchema, PlanarEditBinding<ExampleSchema>> for EditCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "invariant-read-topology-edits.source-completion.v1";
}
