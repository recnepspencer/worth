//! Shared non-workflow Directory Home correction fixture (`LabelEdit` / `RestoreLabel`)
//! for C4b adversarial courts.
use super::{build_id, BUILD_PRIMARY};
use std::time::{Duration, Instant};
use worth_execution::ExecutionRequest;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionSetup,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationPrincipalKey,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphInstallationDenial,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture as label;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

use label::correction_source::label_edit::{LabelBinding, LabelHandler};
use label::{schema_declaration::IdentityExecutionContribution, IdentityExecutionSchema};

pub(super) struct AdvLabelProgram;
pub(super) struct AdvLabelSurface;
struct AdvLabelFeature;
struct AdvLabelDelivery;

impl ApplicationSchemaContribution<IdentityExecutionSchema> for AdvLabelSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <IdentityExecutionContribution as ApplicationSchemaContribution<IdentityExecutionSchema>>::IDENTITY;
    type Members = <IdentityExecutionContribution as ApplicationSchemaContribution<
        IdentityExecutionSchema,
    >>::Members;
}
impl WorthQueryApplicationContribution<IdentityExecutionSchema> for AdvLabelSurface {
    type Configuration = ();
    fn configure(
        _: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, IdentityExecutionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<LabelBinding, _>(LabelHandler)?;
        setup.handler::<label::program_required_binding::ProgramRequiredMutationBinding, _>(
            label::program_required_binding::ProgramRequiredHandler,
        )?;
        setup.handler::<label::capability_touch_binding::CapabilityTouchMutationBinding, _>(
            label::capability_touch_binding::CapabilityTouchHandler,
        )?;
        setup.handler::<label::optional_output_binding::OptionalOutputMutationBinding, _>(
            label::optional_output_binding::OptionalOutputHandler,
        )
    }
}
impl ApplicationFeature<IdentityExecutionSchema> for AdvLabelFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.store.court.adv.label.feature.v1";
}
impl ApplicationActionDelivery<IdentityExecutionSchema, LabelBinding> for AdvLabelDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.store.court.adv.label.completion.v1";
}
impl ApplicationProgramDefinition<IdentityExecutionSchema> for AdvLabelProgram {
    type Contributions = (AdvLabelSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.store.court.adv.label.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, AdvLabelFeature>()
                .action::<LabelBinding, AdvLabelDelivery>()
                .finish_action()
                .native_action::<label::program_required_binding::ProgramRequiredMutationBinding>()
                .finish_action()
                .finish(),
        ]
    }
}

pub(super) type LabelHome = WorthQueryApplicationHome<IdentityExecutionSchema, AdvLabelProgram>;

pub(super) fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(120)),
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    )
}

pub(super) fn open_label_home(
    location: Location,
    placement: Option<AdversarialPlacement>,
    execution: ExecutionRequest<'_, '_>,
) -> LabelHome {
    open_label_home_with_build(location, BUILD_PRIMARY, placement, execution)
}

pub(super) fn open_label_home_with_build(
    location: Location,
    build: &str,
    placement: Option<AdversarialPlacement>,
    execution: ExecutionRequest<'_, '_>,
) -> LabelHome {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let mut req = WorthQueryApplicationHomeRequest::new(
        location,
        ApplicationProgramAuthoring::<IdentityExecutionSchema, AdvLabelProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build_id(build),
        worth_query_testkit_document_retention::facade::document_retention_model::host::host_limits(
            WorthQueryAdmissionClock::system(),
        ),
        |_, graph, schema| {
            let binding = schema
                .principal_binding(label::IdentityBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                1,
                label::schema_types::external_identity("alice"),
                worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            label::account_seed::bind_account(
                graph,
                label::account_seed::AccountSeedSpec {
                    key: "account-1",
                    status: "open",
                    label: "primary",
                    note: None,
                },
            );
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                label::AccountOwner::reference(),
                "account-owner",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))?;
            Ok(())
        },
    );
    if let Some(p) = placement {
        req = host_observation::opening::attach_placement(req, p).0;
    }
    (*with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(req.open_in_advancement(&phase))
    })
    .unwrap())
    .unwrap_or_else(|e| panic!("correction Home open: {e:?}"))
}
