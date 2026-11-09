//! Compile configured producers and conditional providers once before Signal admission.
use super::super::*;
use super::assembly;
use super::{authored_bindings::QueryHomeAuthoredBindings, program_state::HomeServingProgramState};
use crate::domain_computation::primary_graph::{
    application_contribution::*, application_runtime::installation::home_attachment,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, RuntimeBridge, RuntimeBridgeRelationalSource,
};
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) struct HomeServingProgramPreparation<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) state:
        HomeServingProgramState<Schema, Program>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bootstrap:
        WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) services:
        home_attachment::HomeApplicationServices<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) ordinary:
        RuntimeBridge,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) installation:
        worth_proof::BridgeProgramInstallationBinding,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) producers:
        WorthQueryInstalledApplicationProducerRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) conditionals:
        WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) producer_routes:
        WorthQueryInstalledOutputProducerRoutes,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) readiness_routes:
        WorthQueryInstalledOutputReadinessRoutes,
}
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) struct HomeSignalProgramParts<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) prepared:
        HomeServingProgramPreparation<Schema, Program>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) builder:
        Box<BridgeConditionalRuntimeBuilder>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bindings:
        QueryHomeAuthoredBindings,
}
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn prepare<
    Schema: ApplicationSchemaComposition,
    Program,
>(
    phase: &super::super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    state: Box<HomeApplicationState<Schema, Program>>,
    schema: Box<InstalledNativeSchemaMeaning>,
    restore: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    source: RuntimeBridgeRelationalSource,
    ordinary: RuntimeBridge,
    seed: Option<super::super::super::super::bootstrap::WorthQueryProgramActivationSeed>,
) -> Result<
    HomeSignalProgramParts<Schema, Program>,
    super::program_preparation_custody::QueryHomeSignalProgramPreparationRefused<Schema, Program>,
> {
    use super::program_preparation_custody::{
        HomeProgramInstallationCustody, NativeProgramInput,
        QueryHomeSignalProgramPreparationRefused,
    };
    let input = Box::new(NativeProgramInput {
        state,
        schema,
        restore,
        source,
        ordinary,
        seed,
    });
    let owner = match super::admit_registered_source(
        &input.state,
        &input.schema,
        &input.source,
        &input.ordinary,
    ) {
        Ok(owner) => owner,
        Err(cause) => {
            return Err(QueryHomeSignalProgramPreparationRefused::native(
                input, cause,
            ))
        }
    };
    let NativeProgramInput {
        state,
        schema,
        restore,
        source,
        ordinary,
        seed,
    } = *input;
    let (state, mut bootstrap) = assembly::prepare(*state, *schema, restore, owner);
    bootstrap.program_activation_seed = seed;
    let mut custody =
        HomeProgramInstallationCustody::from_meaning(state, bootstrap, source, ordinary);
    use super::program_preparation_denial::QueryHomeSignalProgramPreparationDenial as D;
    let result = (|| -> Result<(), D> {
        let ctx = &mut *custody;
        ctx.install_contributions(phase)?;
        ctx.prepare_services(phase)?;
        let budget = ctx.conditional_budget();
        ctx.builder = Some(Box::new(
            BridgeConditionalRuntimeBuilder::with_owned_signal_graph(ctx.ordinary.clone(), budget)
                .map_err(|cause| D::Bridge(Box::new(cause)))?,
        ));
        let services = ctx.services.as_ref().expect("actual prepared services");
        let builder = ctx.builder.as_mut().expect("actual programme builder");
        let producers = ctx
            .producers
            .as_ref()
            .expect("original configured producers");
        ctx.producer_routes = Some(
            producers
                .install_signal_routes(builder)
                .map_err(D::PrimaryGraph)?,
        );
        ctx.readiness_routes = Some(
            install_output_readiness_routes(&ctx.readiness, builder, &services.participation)
                .map_err(|cause| D::Conditional(Box::new(cause)))?,
        );
        producers
            .validate_readiness_routes(
                ctx.readiness_routes
                    .as_ref()
                    .expect("actual readiness routes"),
            )
            .map_err(D::PrimaryGraph)?;
        services
            .provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .install_output_families(producers.output_family_bindings());
        ctx.bindings = Some(
            QueryHomeAuthoredBindings::from_installed_program(builder, &phase.execution_request())
                .map_err(|cause| D::Authored(Box::new(cause)))?,
        );
        ctx.bindings
            .as_ref()
            .expect("actual fresh bindings")
            .admit_routes(
                ctx.producer_routes
                    .as_mut()
                    .expect("actual producer routes"),
                ctx.readiness_routes
                    .as_mut()
                    .expect("actual readiness routes"),
            )
            .map_err(|cause| D::Authored(Box::new(cause)))?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(custody.finish()),
        Err(cause) => Err(custody.refuse(cause)),
    }
}
