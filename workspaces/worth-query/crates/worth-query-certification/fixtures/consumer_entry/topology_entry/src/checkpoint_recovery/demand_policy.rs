use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
    ApplicationFeature, ApplicationFeatureSpecBuilder, ApplicationLocalityGranule,
    ApplicationLocalityScope, ApplicationRootComposition,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
};
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

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
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4_096, 8_192);
    const STOPPED_OUTCOME: &'static str = "final-output-resource-denied";
}

#[test]
fn installed_large_parent_allowance_composes_with_small_child_artifact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut output = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .expect("broad parent allowance admits its actual smaller child");
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
        .expect("default host allowances admit initial production");
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

#[test]
fn restrictive_host_denies_estimate_before_operation_or_publication() {
    let _guard = checkpoint_recovery_test_guard();
    let profile = WorthQueryOutputDemandResourceProfile::bounded(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(4_095).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
        NonZeroUsize::new(64).unwrap(),
    );
    let application = support::install_with_demand_profile(None, profile);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = request.retain_read().unwrap();
    super::super::producer::reset_provider_contacts();
    let denied = match request.demand(PlanarOutputDemand::new("anchor-a")).start() {
        Ok(_) => panic!("the producer estimate exceeds host work capacity"),
        Err(denied) => denied,
    };
    assert!(
        matches!(denied, WorthQueryApplicationOutputDemandDenial::Demand(cause)
        if cause.kind() == WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
    );
    // A fresh estimate is one provider callback; operation_input/idempotency
    // are not contacted and no mutation is published.
    assert_eq!(super::super::producer::provider_contacts(), 1);
    let after = request.retain_read().unwrap();
    assert_eq!(before.selected_commit(), after.selected_commit());
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
            .mutation::<PlanarEditBinding<CheckpointSchema>>()
            .finish(),
        output,
        final_output,
        ApplicationFeatureSpec::root::<CheckpointSchema, PlanarAlternateFinalOutputFeature>()
            .provides::<PlanarAlternateFinalBodyOutput>()
            .conditional_operation::<PublishAlternatePlanarOutput>()
            .finish(),
    ]
}

struct InsufficientFinalArtifact;
impl ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature>
    for InsufficientFinalArtifact
{
    type Output = PlanarFinalBodyOutput;
    type Locality = PlanarLocality;
    const IDENTITY: &'static str = "checkpoint-insufficient-final-planar-artifact";
    const RETENTION: ApplicationArtifactRetention = FinalArtifact::RETENTION;
    const SUCCESSION: ApplicationArtifactSuccession = FinalArtifact::SUCCESSION;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = FinalArtifact::PRODUCER_FAMILY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = FinalArtifact::DEPENDENCIES;
    const REUSE_RULE: &'static str = FinalArtifact::REUSE_RULE;
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4_095, 8_192);
    const STOPPED_OUTCOME: &'static str = "final-output-resource-denied";
}

struct InsufficientChildProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for InsufficientChildProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-insufficient-child-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        feature_specs::<InsufficientFinalArtifact>()
    }
}

#[test]
fn installed_child_excess_is_denied_without_publishing_the_child() {
    let _guard = checkpoint_recovery_test_guard();
    let application =
        support::install_program::<InsufficientChildProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut output = request
        .start_program_outputs::<InsufficientChildProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .expect("the sufficient parent starts independently of its child");
    // The call that settles the parent starts the child and is refused. The
    // refusal is kept: the next call starts the child again, is refused
    // again and publishes nothing.
    let mut before = request.retain_read().unwrap();
    for call in ["the call that settles the parent", "the next call"] {
        match output.advance(&application, &request) {
            Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Demand(denial),
            )) if denial.kind() == WorthQueryOutputDemandDenialKind::WorkBudgetExceeded => {}
            other => panic!(
                "{call}: an oversized child is refused its work: {:?}",
                other.map(|progress| matches!(
                    progress,
                    WorthQueryApplicationProgramOutputProgress::Settled(_)
                ))
            ),
        }
        let after = request.retain_read().unwrap();
        let (before_commit, after_commit) = (before.selected_commit(), after.selected_commit());
        if call == "the next call" {
            assert_eq!(
                before_commit, after_commit,
                "{call}: a refused child publishes nothing"
            );
        } else {
            assert_ne!(
                before_commit, after_commit,
                "{call}: the parent is published"
            );
        }
        before = after;
    }
    let parent_read = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the parent output is published");
    assert!(
        !parent_read.rows().is_empty(),
        "the parent output is readable while its child is refused"
    );
    let child_read = request
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".to_owned(),
        })
        .execute();
    assert!(matches!(child_read,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial))
            if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::UnknownEntity
                && denial.subject() == "BodyKey"
    ));
    let selected_child =
        request
            .demand(PlanarFinalOutputDemand::new("anchor-a"))
            .start_dependent_in_program::<InsufficientChildProgram, FinalConnection>(&application);
    assert!(
        matches!(selected_child,
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        ),
        "direct selected-program demand must not bypass the artifact ceiling"
    );
}
