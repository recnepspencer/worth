use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, UiMountedFrameRetentionBudget, WorthUiApp,
    WorthUiApplicationBuilder, WorthUiHostNeutralApp,
};
use worth_ui_runtime::facade::entry::UiIntentProviderRequired;

use super::fixed_host::FixedCertificationHostBinding;

type HostNeutralBuilder =
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>;
type Activation = Box<dyn FnOnce(WorthUiHostNeutralApp) -> WorthUiApp>;

pub struct FixedCertificationIntentProviderBuilder<I: worth_ui::facade::intent::UiIntent> {
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentProviderRequired<I>>,
    activation: Activation,
}

/// Certification-owned higher transition. Builder mutation remains entirely
/// host-neutral; the fixed host is activated only after successful freeze.
pub struct FixedCertificationApplicationBuilder {
    builder: HostNeutralBuilder,
    activation: Activation,
}

impl FixedCertificationApplicationBuilder {
    pub fn new<Host>(builder: HostNeutralBuilder, host: Host) -> Self
    where
        Host: FixedCertificationHostBinding,
    {
        Self {
            builder,
            activation: Box::new(move |application| host.activate(application)),
        }
    }

    pub fn freeze(
        self,
    ) -> Result<WorthUiApp, worth_ui::facade::app::WorthUiApplicationPreparationDenial> {
        let application = self.builder.freeze()?;
        Ok((self.activation)(application))
    }

    pub(crate) fn map_builder(
        self,
        transition: impl FnOnce(HostNeutralBuilder) -> HostNeutralBuilder,
    ) -> Self {
        Self {
            builder: transition(self.builder),
            activation: self.activation,
        }
    }

    pub fn register_component(
        self,
        descriptor: worth_ui::facade::declaration::ComponentDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_component(descriptor))
    }

    pub fn register_surface(
        self,
        descriptor: worth_ui::facade::declaration::SurfaceDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_surface(descriptor))
    }

    pub fn register_command(
        self,
        descriptor: worth_ui::facade::declaration::CommandDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_command(descriptor))
    }

    pub fn register_mosaic_region_kind(
        self,
        descriptor: worth_ui::facade::declaration::MosaicRegionKindDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_mosaic_region_kind(descriptor))
    }

    pub fn register_mosaic_sizing_contract(
        self,
        descriptor: worth_ui::facade::declaration::MosaicSizingContractDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_mosaic_sizing_contract(descriptor))
    }

    pub fn register_mosaic_state_slot(
        self,
        descriptor: worth_ui::facade::declaration::MosaicStateSlotDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_mosaic_state_slot(descriptor))
    }

    pub fn with_candidate_submission(
        self,
        submission: worth_ui::facade::source::WorthUiWatchedCandidateSubmission,
    ) -> Self {
        self.map_builder(|builder| builder.with_candidate_submission(submission))
    }

    pub fn with_mounted_frame_retention_budget(
        self,
        budget: UiMountedFrameRetentionBudget,
    ) -> Self {
        self.map_builder(|builder| builder.with_mounted_frame_retention_budget(budget))
    }

    pub fn with_host_observation_capacity(
        self,
        capacity: worth_ui_runtime::facade::observation_report::UiHostObservationCapacity,
    ) -> Self {
        self.map_builder(|builder| builder.with_host_observation_capacity(capacity))
    }

    pub fn with_visual_inspection_policy(
        self,
        policy: worth_ui::facade::inspection::UiVisualInspectionPolicy,
    ) -> Self {
        self.map_builder(|builder| builder.with_visual_inspection_policy(policy))
    }

    pub fn with_command_routing_policy_defaults(
        self,
        policy: worth_ui::facade::service::UiCommandRoutingPolicy,
    ) -> Self {
        self.map_builder(|builder| builder.with_command_routing_policy_defaults(policy))
    }

    pub fn with_selection_policy_defaults(
        self,
        policy: worth_ui::facade::service::UiSelectionPolicy,
    ) -> Self {
        self.map_builder(|builder| builder.with_selection_policy_defaults(policy))
    }

    pub fn with_runtime_instance_basis_admissions(
        self,
        admissions: impl IntoIterator<Item = worth_ui::facade::graph::UiRuntimeInstanceBasisAdmission>,
    ) -> Self {
        use worth_ui_test_support::WorthUiApplicationBuilderCertificationExt;
        self.map_builder(|builder| builder.with_runtime_instance_basis_admissions(admissions))
    }

    pub fn with_rust_authored_input(
        self,
        input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
    ) -> Self {
        self.map_builder(|builder| builder.with_rust_authored_input(input))
    }

    pub fn register_intent_boolean_fact(
        self,
        fact: worth_ui::facade::intent::UiIntentApplicationFact<
            worth_ui::facade::intent::UiIntentBoolean,
        >,
        initial: bool,
    ) -> Result<Self, worth_ui::facade::intent::UiIntentApplicationFactRegistrationError> {
        let builder = self.builder.register_intent_boolean_fact(fact, initial)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_theme_token(
        self,
        descriptor: worth_ui::facade::declaration::ThemeTokenDescriptor,
    ) -> Self {
        self.map_builder(|builder| builder.register_theme_token(descriptor))
    }

    pub fn register_appearance_role(
        self,
        role: worth_ui::facade::appearance::UiAppearanceRoleDeclaration,
    ) -> Result<Self, worth_ui::facade::appearance::AppearanceRoleRegistrationDenial> {
        let builder = self.builder.register_appearance_role(role)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_appearance_theme_bundle(
        self,
        bundle: worth_ui::facade::appearance::FrozenAppearanceThemeCapabilities,
    ) -> Result<Self, worth_ui::facade::appearance::FrozenAppearanceThemeCapabilitiesDenial> {
        let builder = self.builder.register_appearance_theme_bundle(bundle)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_intent_text_fact(
        self,
        fact: worth_ui::facade::intent::UiIntentApplicationFact<
            worth_ui::facade::intent::UiIntentText,
        >,
        initial: impl Into<std::sync::Arc<str>>,
    ) -> Result<Self, worth_ui::facade::intent::UiIntentApplicationFactRegistrationError> {
        let builder = self.builder.register_intent_text_fact(fact, initial)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_intent_unsigned64_fact(
        self,
        fact: worth_ui::facade::intent::UiIntentApplicationFact<
            worth_ui::facade::intent::UiIntentUnsigned64,
        >,
        initial: u64,
    ) -> Result<Self, worth_ui::facade::intent::UiIntentApplicationFactRegistrationError> {
        let builder = self
            .builder
            .register_intent_unsigned64_fact(fact, initial)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_query_view(
        self,
        registration: impl Into<
            worth_ui_runtime::facade::registry::descriptor::WorthUiQueryViewRegistration,
        >,
    ) -> Result<Self, worth_ui::facade::query_binding::WorthUiQueryViewRegistrationError> {
        let builder = self.builder.register_query_view(registration)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_scalar_projection(
        self,
        registration: worth_ui::facade::query_binding::UiScalarProjectionRegistration,
    ) -> Result<Self, worth_ui::facade::query_binding::WorthUiProjectionRegistrationError> {
        let builder = self.builder.register_scalar_projection(registration)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_application_scalar_projection(
        self,
        registration: worth_ui::facade::query_binding::UiApplicationScalarProjectionRegistration,
    ) -> Result<Self, worth_ui::facade::query_binding::WorthUiProjectionRegistrationError> {
        let builder = self
            .builder
            .register_application_scalar_projection(registration)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_collection_projection(
        self,
        registration: worth_ui::facade::query_binding::UiCollectionProjectionRegistration,
    ) -> Result<Self, worth_ui::facade::query_binding::WorthUiProjectionRegistrationError> {
        let builder = self.builder.register_collection_projection(registration)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_intent_transition_definition<I>(
        self,
        definition: worth_ui::facade::intent::UiIntentDefinition<
            I,
            worth_ui::facade::intent::UiTransitionDefinitionDestination,
        >,
    ) -> Result<Self, worth_ui::facade::intent::UiIntentDefinitionRegistrationError>
    where
        I: worth_ui::facade::intent::UiIntent,
        I::ProductOutcome: worth_ui::facade::intent::UiIntentTransitionOutcome,
    {
        let builder = self
            .builder
            .register_intent_transition_definition(definition)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_runtime_service_intent_definition<I>(
        self,
        definition: worth_ui::facade::intent::UiIntentDefinition<
            I,
            worth_ui::facade::intent::UiRuntimeServiceDefinitionDestination,
        >,
    ) -> Result<Self, worth_ui::facade::intent::UiIntentDefinitionRegistrationError>
    where
        I: worth_ui::facade::intent::UiIntent,
    {
        let builder = self
            .builder
            .register_runtime_service_intent_definition(definition)?;
        Ok(Self::from_parts(builder, self.activation))
    }

    pub fn register_intent_definition<I>(
        self,
        definition: worth_ui::facade::intent::UiIntentDefinition<I>,
    ) -> Result<
        FixedCertificationIntentProviderBuilder<I>,
        worth_ui::facade::intent::UiIntentDefinitionRegistrationError,
    >
    where
        I: worth_ui::facade::intent::UiIntent,
    {
        let builder = self.builder.register_intent_definition(definition)?;
        Ok(FixedCertificationIntentProviderBuilder {
            builder,
            activation: self.activation,
        })
    }

    pub(crate) fn from_parts(builder: HostNeutralBuilder, activation: Activation) -> Self {
        Self {
            builder,
            activation,
        }
    }
}

impl<I: worth_ui::facade::intent::UiIntent> FixedCertificationIntentProviderBuilder<I> {
    pub fn register_intent_provider<Provider>(
        self,
        provider: Provider,
    ) -> Result<
        FixedCertificationApplicationBuilder,
        worth_ui::facade::intent::UiIntentExecutionBindingPreparationDenial,
    >
    where
        Provider: worth_ui::facade::intent::UiIntentExecutionProvider<I>,
    {
        let builder = self.builder.register_intent_provider(provider)?;
        Ok(FixedCertificationApplicationBuilder::from_parts(
            builder,
            self.activation,
        ))
    }
}
