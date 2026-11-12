//! Selected authored definitions open the same durable Signal-only World intent.
use super::*;
use worth_runtime_world::facade::WorldOpenAttemptOutcome;

pub(super) fn publish<
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    product: &WorthQueryProductBranchLease,
    bridge: &Arc<worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly>,
    predecessor: &Arc<BridgeInstalledConditionalLowering>,
    request: BridgeOwnedConditionalInstallationRequest,
    cancellation: &RuntimeWorldCancellationToken,
) -> Result<
    RuntimeWorldConditionalDefinitionPublicationOutcome,
    WorthQueryProductConditionalPublicationDenial,
> {
    use WorthQueryProductConditionalPublicationDenial as D;
    let execution = phase
        .execution_request_for(&application.runtime.product_runtime)
        .map_err(|cause| D::ExecutionRequest(cause.into()))?;
    let branch = worth_foundational::facade::DurableSignalBranchIdentity::from_recorded(
        bridge.signal_owner().binding().owner(),
        product.signal_basis().branch_id().0,
    );
    let definition = bridge
        .prepare_conditional_definition_publication(&execution, branch, predecessor, request)
        .map_err(D::AuthoredBridgePreparation)?;
    let product_binding = product.publication_binding();
    let publication = application.runtime.product_runtime.owner.publication_port();
    let mut pending = publication
        .prepare_query_publication_with_signal(
            &execution,
            product.observation().clone(),
            product_binding
                .admitted_program_source()
                .map_err(D::DurableOpen)?,
            None,
            cancellation,
            None,
        )
        .map_err(D::DurableOpen)?;
    let opened = loop {
        match pending.advance(&execution) {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(original) => {
                if execution.consult().is_err() || cancellation.is_cancelled() {
                    return Ok(
                        RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(
                            original.retain_conditional_definition(),
                        ),
                    );
                }
                pending = original;
            }
            WorldOpenAttemptOutcome::Unresolved(original)
            | WorldOpenAttemptOutcome::EvidenceRequired(original) => {
                return Ok(
                    RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(
                        original.retain_conditional_definition(),
                    ),
                )
            }
            WorldOpenAttemptOutcome::Refused(cause) => return Err(D::DurableOpen(cause)),
        }
    };
    let prepared = opened
        .prepare_signal_only(&execution)
        .map_err(D::DurablePreparation)?;
    Ok(publication.publish_durable_bridge_conditional_definition(
        execution,
        prepared,
        definition,
        cancellation,
    ))
}
