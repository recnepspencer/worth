use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactRetention, ApplicationArtifactSuccession,
    ApplicationDerivedArtifact, ApplicationFeature, ApplicationFeatureSpecBuilder,
    ApplicationLocalityGranule, ApplicationLocalityScope, ApplicationRootComposition,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress;

pub(super) struct FinalArtifact;
pub(super) struct PlanarLocality;

impl ApplicationLocalityScope for PlanarLocality {
    const IDENTITY: &'static str = "checkpoint-planar-occurrence";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}

impl ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature> for FinalArtifact {
    type Output = PlanarFinalBodyOutput;
    type Locality = PlanarLocality;
    const IDENTITY: &'static str = "checkpoint-final-planar-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession =
        ApplicationArtifactSuccession::PreserveWhenEquivalent;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <PlanarFinalOutputFamily as WorthQueryProducerOutputFamily<CheckpointSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarOutputFeature as ApplicationFeature<CheckpointSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";

    const STOPPED_OUTCOME: &'static str = "final-output-resource-denied";
}

#[test]
fn installed_parent_produces_required_child_artifact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut output = request
        .advanced()
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .expect("installed parent starts with its required child");
    let settled = (0..256)
        .find_map(|_| match output.advance(&application, &request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        })
        .expect("all installed child outputs settle");
    assert_eq!(
        settled
            .outputs_for::<CheckpointSchema, PlanarOutputToFinalConnection>()
            .count(),
        1
    );
}

#[test]
fn omitted_controls_produce_and_reuse_real_output_under_installed_defaults() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut initial = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("installed defaults admit initial production");
    let first = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("default initial demand settles");
    assert_eq!(first.producer_contacts_in_this_demand(), 1);
    drop(first);
    drop(initial);
    super::super::producer::reset_provider_contacts();
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start()
        .expect("no application resource arithmetic is needed");
    let settled = (0..64)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("default demand settles");
    assert_eq!(settled.producer_contacts_in_this_demand(), 0);
    assert_eq!(super::super::producer::provider_contacts(), 0);
}

pub(super) fn feature_specs<Artifact>() -> Vec<ApplicationFeatureSpec>
where
    Artifact: ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature>,
{
    feature_specs_with_output::<Artifact>(required_chain::output_feature_spec())
}

pub(super) fn feature_specs_with_output<Artifact>(
    output: ApplicationFeatureSpec,
) -> Vec<ApplicationFeatureSpec>
where
    Artifact: ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature>,
{
    feature_specs_with_final_output(output, final_output_feature::<Artifact>().finish())
}

/// The final output feature before it is finished, so a program can attach
/// more members to it.
pub(super) fn final_output_feature<Artifact>() -> ApplicationFeatureSpecBuilder<
    CheckpointSchema,
    ApplicationRootComposition,
    PlanarFinalOutputFeature,
>
where
    Artifact: ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature>,
{
    ApplicationFeatureSpec::root::<CheckpointSchema, PlanarFinalOutputFeature>()
        .derived_artifact::<Artifact>()
        .conditional_operation::<PublishFinalPlanarOutput>()
        .conditional_operation::<PreserveFinalPlanarOutput>()
}

pub(super) fn feature_specs_with_final_output(
    output: ApplicationFeatureSpec,
    final_output: ApplicationFeatureSpec,
) -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<CheckpointSchema, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .native_action::<PlanarEditBinding<CheckpointSchema>>()
            .finish_action()
            .finish(),
        output,
        final_output,
        ApplicationFeatureSpec::root::<CheckpointSchema, PlanarAlternateFinalOutputFeature>()
            .provides::<PlanarAlternateFinalBodyOutput>()
            .conditional_operation::<PublishAlternatePlanarOutput>()
            .finish(),
    ]
}
