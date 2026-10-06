//! The one installation path every open runs, from package admission to a
//! published application runtime.

use super::home_opening::HomeStarted;
use super::home_start::{self, AdmittedEntry};
use super::open_plan::{HomeStart, OpenEntry, OpenPlan};
use super::open_refusal::OpenFailure;
use super::{checkpoint_lineage, WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryApplicationContributionTuple, WorthQueryConfiguredApplicationContributions,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryReadmittedAcceptedOutput, WorthQueryRecoveredOutputs,
};
use crate::domain_computation::primary_graph::invariant_installation::WorthQueryInvariantProgramBasis;
use crate::domain_computation::primary_graph::program_occurrence::{
    WorthQueryInstalledProgramSupport, WorthQueryProgramActivationCell,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub(in crate::domain_computation::primary_graph) fn open_with_contributions<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    plan: OpenPlan<'_, Schema>,
) -> Result<
    (
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        HomeStarted,
    ),
    OpenFailure,
>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationOpenDenial as Denial;

    let OpenPlan {
        entry,
        start,
        authorization_time_source,
    } = plan;
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ));
    let contracts = Contributions::contracts().map_err(Denial::Contributions)?;
    let package = contracts
        .compose_package(package.application_schema(declaration.clone()))
        .validate()
        .map_err(Denial::Package)?;
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "primary-graph-in-memory",
        "application-contributions",
    )
    .admit(package)
    .map_err(Denial::Admission)?;
    let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
        .application_candidate_resources(limits.candidates)
        .application_query_resources(limits.queries)
        .output_demand_resources(limits.output_demands)
        .completed_evidence_resources(limits.completed_evidence)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .map_err(Denial::Runtime)?
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .map_err(Denial::Schema)?;
    let entry = match entry {
        OpenEntry::Declaration => {
            if installed
                .installed_mutation_binding_inventory()
                .any(|binding| binding.requires_workflow_authority())
            {
                return Err(Denial::WorkflowAuthorityRequiresProgram.into());
            }
            AdmittedEntry::Declaration
        }
        OpenEntry::Program(admit) => AdmittedEntry::Program(admit(&installed)?),
    };
    let configured = WorthQueryConfiguredApplicationContributions::<Schema>::configure::<
        Contributions,
    >(&installed, configuration, contracts)
    .map_err(Denial::Contributions)?;
    let (mut invariants, handlers, producers, conditionals) =
        configured.into_parts().map_err(Denial::Contributions)?;
    let activation = WorthQueryProgramActivationCell::unpublished();
    if let AdmittedEntry::Program(support) = &entry {
        invariants.select_by_program(WorthQueryInvariantProgramBasis::admitted(
            std::sync::Arc::clone(&support.roster),
            activation.clone(),
        ));
    }
    let mut relational_builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .profile(limits.profile.relational_profile());
    if let Some(publication) = limits
        .profile
        .publication_override(limits.maximum_publication_records)
    {
        relational_builder = relational_builder.publication(publication);
    }
    if let Some(scope_budget) = limits.profile.relation_integrity_scope_budget() {
        relational_builder = relational_builder.relation_integrity_scope_budget(scope_budget);
    }
    let relational_runtime = relational_builder.build();
    let (graph, started, accepted_outputs) = match start {
        HomeStart::Empty { initial_state } => {
            let mut graph = authority
                .prepare_primary_graph_with_relational_runtime_and_invariants(
                    &runtime,
                    &installed,
                    relational_runtime,
                    limits.world,
                    invariants,
                )
                .map_err(Denial::Graph)?;
            graph.mutation_handlers = handlers;
            home_start::seed_empty(&mut graph, &installed, &entry, &activation, initial_state)?;
            (graph, HomeStarted::Empty, Vec::new())
        }
        HomeStart::Resume { image, adoption } => {
            let image = image
                .decode()
                .map_err(|detail| Denial::Graph(recovery_rejected(detail)))?;
            let mut graph = authority
                .prepare_primary_graph_from_native_checkpoint_with_invariants(
                    &runtime,
                    &installed,
                    relational_runtime,
                    limits.world,
                    invariants,
                    &image,
                )
                .map_err(Denial::Graph)?;
            graph.mutation_handlers = handlers;
            let started = home_start::resume(
                &mut graph,
                &installed,
                &entry,
                &activation,
                adoption,
                !image.accepted_outputs.is_empty(),
            )?;
            (graph, started, image.accepted_outputs)
        }
    };
    // Every step below runs after the home started. A refusal there returns
    // the successor once an adoption was acknowledged.
    let published = (|| -> Result<_, Denial> {
        // This is the exact support object moved into the final graph provider.
        // Cold producer executors become installable only after it exists.
        let producers = producers
            .seal_with_support(graph.resource_support_ref(), runtime.installed_packages())
            .map_err(Denial::Contributions)?;
        let (mut application, installed_conditionals) = if conditionals.is_empty()
            && producers.is_empty()
        {
            let application = match authorization_time_source {
                Some(source) => graph.publish_application_runtime_with_authorization_time_source(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                    source,
                ),
                None => graph.publish_application_runtime(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                ),
            }
            .map_err(Denial::Publication)?;
            (application, Default::default())
        } else {
            let mut publication = match authorization_time_source {
                Some(source) => graph
                    .conditional_application_runtime_installation_with_authorization_time_source(
                        runtime,
                        authority,
                        installed,
                        limits.conditionals,
                        source,
                    ),
                None => graph.conditional_application_runtime_installation(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                ),
            }
            .map_err(Denial::ConditionalPublication)?;
            publication.install_output_producers(producers.clone());
            let installed_conditionals = conditionals
                .install_all(&producers, &mut publication)
                .map_err(Denial::ConditionalPublication)?;
            let application = publication
                .publish()
                .map_err(Denial::ConditionalPublication)?;
            (application, installed_conditionals)
        };
        application.installed_producers = producers;
        application.installed_conditionals = installed_conditionals;
        if let AdmittedEntry::Program(support) = entry {
            application
                .product_runtime
                .activations
                .require_program_coordination();
            application.program_support = Some(WorthQueryInstalledProgramSupport::installed(
                support.roster,
                activation,
            ));
        }
        application.recovered_outputs = WorthQueryRecoveredOutputs::from_records(
            accepted_outputs
                .into_iter()
                .map(|accepted| {
                    let correspondence = application
                        .installed_producers
                        .readmit_checkpoint_output(&application.installed_schema, &accepted)
                        .map_err(|detail| Denial::Graph(recovery_rejected(detail)))?;
                    Ok(WorthQueryReadmittedAcceptedOutput {
                        checkpoint: accepted,
                        correspondence,
                    })
                })
                .collect::<Result<Vec<_>, Denial>>()?,
        );
        checkpoint_lineage::restore(&application).map_err(Denial::Graph)?;
        Ok(application)
    })();
    match published {
        Ok(application) => Ok((application, started)),
        Err(denial) => Err(started.refused(denial)),
    }
}

fn recovery_rejected(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        detail,
    )
}
