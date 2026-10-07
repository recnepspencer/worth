//! Readmit retained outputs against the published schema and producers.
use super::super::{
    home_opening::HomeStarted, open_refusal::OpenFailure, WorthQueryApplicationOpenDenial as Denial,
};
use super::publication::PublishedInstallation;
use crate::domain_computation::primary_graph::{
    application_output_demand::{WorthQueryReadmittedAcceptedOutput, WorthQueryRecoveredOutputs},
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) struct ReadmittedInstallation<Schema> {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) started: HomeStarted,
}

pub(super) fn readmit<Schema: ApplicationSchema>(
    installation: PublishedInstallation<Schema>,
) -> Result<ReadmittedInstallation<Schema>, OpenFailure> {
    let PublishedInstallation {
        mut application,
        started,
        accepted_outputs,
    } = installation;
    let recovered = (|| -> Result<(), Denial> {
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
        Ok(())
    })();
    match recovered {
        Ok(()) => Ok(ReadmittedInstallation {
            application,
            started,
        }),
        Err(denial) => Err(started.refused(denial)),
    }
}

fn recovery_rejected(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        detail,
    )
}
