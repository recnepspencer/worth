//! A producer's complete port contract is checked before Native bootstrap.
use super::super::super::synchronous_edit::{envelope_fixture as fixture, interleaved};
use super::*;

struct MissingAnchor;
impl ApplicationOccurrenceComponent<Schema> for MissingAnchor {
    type Connections = (ApplicationCapturedInputConnection<program::SeedConnection, Capture>,);
}
type IncompleteTree =
    ApplicationOutputGraph<program::Root, ApplicationOutputOccurrences<MissingAnchor>>;
struct IncompleteProgram;
impl ApplicationProgramDefinition<Schema> for IncompleteProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<IncompleteTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.incomplete");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<IncompleteTree>>()
    }
}

#[test]
fn omitting_one_declared_capture_port_refuses_before_bootstrap() {
    let declaration = Schema::declaration().unwrap();
    let validated = ApplicationProgramAuthoring::<Schema, IncompleteProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let configuration = world::Configuration {
        envelope_records: true,
        interleaved_records: true,
        captured_inputs: true,
        ..Default::default()
    };
    let bootstrapped = std::cell::Cell::new(false);
    let result = in_memory_program(
        validated,
        declaration,
        (configuration,),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            bootstrapped.set(true);
            world::seed(graph, installed, 1, None)?;
            fixture::seed(graph)?;
            interleaved::seed(graph)?;
            program::seed(graph, None)
        },
    );
    let Err(WorthQueryApplicationPreparationDenial::Contributions(cause)) = result else {
        panic!("incomplete capture context was not refused by its installation owner")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::ProducerBindingMeaningMismatch
    );
    assert!(
        !bootstrapped.get(),
        "the real world must not be seeded after a missing-port refusal"
    );
}
