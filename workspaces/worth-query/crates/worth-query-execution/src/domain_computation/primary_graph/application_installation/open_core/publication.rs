//! Publish a started graph and carry its home custody into output recovery.
use super::super::{
    home_opening::HomeStarted, home_start::AdmittedEntry, open_refusal::OpenFailure,
    WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial as Denial,
};
use super::graph_start::StartedInstallation;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    program_occurrence::WorthQueryInstalledProgramSupport,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) struct PublishedInstallation<Schema> {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) started: HomeStarted,
    pub(super) accepted_outputs: Vec<WorthQueryAcceptedOutputCheckpointIdentity>,
}

pub(super) fn publish<Schema: ApplicationSchema>(
    installation: StartedInstallation<Schema>,
    limits: &WorthQueryApplicationLimits,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
) -> Result<PublishedInstallation<Schema>, OpenFailure> {
    let StartedInstallation {
        runtime,
        authority,
        installed,
        entry,
        activation,
        graph,
        producers,
        conditionals,
        started,
        accepted_outputs,
    } = installation;
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
        Ok(application)
    })();
    match published {
        Ok(application) => Ok(PublishedInstallation {
            application,
            started,
            accepted_outputs,
        }),
        Err(denial) => Err(started.refused(denial)),
    }
}
