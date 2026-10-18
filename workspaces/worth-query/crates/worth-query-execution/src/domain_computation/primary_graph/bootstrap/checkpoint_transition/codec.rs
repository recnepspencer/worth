//! Join authenticated Query activation to the exclusive native codec owner.
use super::denial;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    durability::RecoveredRelationalRuntimeAuthority,
    mvcc::BranchBoundRelationalTransaction,
    runtime::{
        RecoveredCheckpointProgramActivationUpdate, RelationalSchemaTransitionAdmissionDenial,
    },
};

pub(in crate::domain_computation::primary_graph::bootstrap) fn map_admission(
    error: RelationalSchemaTransitionAdmissionDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    if let Some(cause) = error.request_cause() {
        return super::super::preparation::map_checkpoint_recovery_stop(
            worth_relational::facade::durability::RelationalRecoveryStop::Request(*cause),
        );
    }
    denial(format!(
        "native codec transition admission refused: {error}"
    ))
}

pub(super) fn begin<Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraphBootstrap<Schema>,
    recovered: &mut RecoveredRelationalRuntimeAuthority,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    activation: RecoveredCheckpointProgramActivationUpdate,
) -> Result<Option<BranchBoundRelationalTransaction>, WorthQueryPrimaryGraphInstallationDenial> {
    if !recovered.codec_transition_pending() {
        return Ok(None);
    }
    let publisher = graph
        .graph
        .application_publication
        .as_ref()
        .ok_or_else(|| denial("native codec transition omitted its source publisher"))?;
    // read_activation authenticated this exact recovered row against the
    // predecessor before this concrete owner proof can be issued.
    graph
        .graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let commitment = runtime
                .durability_recovery()
                .checkpoint_program_activation_commitment(execution, recovered, basis, &activation)
                .map_err(map_admission)?;
            let proof = publisher
                .owner
                .admit_checkpoint_program_activation(commitment);
            runtime
                .durability_recovery()
                .begin_recovered_codec_transition(
                    execution,
                    recovered,
                    basis,
                    worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
                    &publisher.native,
                    proof,
                    activation,
                )
                .map(Some)
                .map_err(map_admission)
        })
}

pub(super) fn map_refusal(
    refused: &worth_relational::facade::durability::RecoveredCheckpointTransitionDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::{
        durability::RecoveredCheckpointTransitionDenial as Refusal,
        indexes::DerivedIndexMaintenanceDenialKind as Index,
        mvcc::RelationalPublicationOutcome as Publication,
    };
    let cause = match refused {
        Refusal::CodecRequest(cause) => Some(*cause),
        Refusal::CodecIndexPreparation(index) => match &index.kind {
            Index::Execution(cause) => Some(*cause),
            _ => None,
        },
        Refusal::Publication(outcome) => match outcome.as_ref() {
            Publication::RequestStopped(stop) => Some((*stop).into()),
            _ => None,
        },
        _ => None,
    };
    if let Some(cause) = cause {
        return super::super::preparation::map_checkpoint_recovery_stop(
            worth_relational::facade::durability::RelationalRecoveryStop::Request(cause),
        );
    }
    denial(format!(
        "checkpoint native transition refused without performance: {refused:?}"
    ))
}

pub(super) fn finish<Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let check_original = || {
        execution
            .consult()
            .map_err(|cause| super::super::request_read_denial::read(cause.into()))
    };
    check_original()?;
    let handle = graph.graph.integration_handle();
    let receipt = handle.with_runtime(execution, |runtime| {
        runtime.readmit_recovered_initial_schema_installation(
            graph.invariant_installation_receipt.clone(),
        )
    });
    check_original()?;
    let receipt = receipt.map_err(|error| {
        denial(format!(
            "codec target initial schema readmission refused: {error}"
        ))
    })?;
    let indexes = super::super::checkpoint::verify_bootstrap_indexes(
        execution,
        &graph.graph,
        handle.primary_index_ids.iter().copied(),
    );
    check_original()?;
    indexes?;
    graph.graph.bind_application_publication(&receipt);
    graph.invariant_installation_receipt = receipt;
    Ok(())
}
