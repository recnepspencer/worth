//! Runtime port implementation for closed certification inspection of committed actor evidence.
use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    worth_query_execution_lineage::facade::ports::WorkflowActorEvidenceRuntimePort
    for crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn retain_workflow_actor_evidence_for_test(
        &self,
        changes: &WorthQueryApplicationCommittedChanges,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &super::super::WorthQueryApplicationCommitReceipt,
        approval: EntityId,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        use crate::domain_computation::authorization::{
            WorkflowApprovalRequestDecodingDenial as Decode, WorthQueryRetainedCapabilityRequest,
        };
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind,
        };
        let invalid = |subject: &str| Denial::new(Kind::InvalidAuthoritativeValue, subject);
        let locator =
            crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                "workflow-approval",
                "authorization-request",
            )
            .map_err(|_| invalid("committed approval request locator is invalid"))?;
        if !changes.shares_original(receipt.committed_changes()) {
            return Err(invalid("committed approval receipt envelope differs"));
        }
        let Some(AspectValue::String(worth_foundational::facade::InternedString::Symbol(symbol))) =
            changes.committed_field_value(approval, &locator)
        else {
            return Err(invalid(
                "committed approval request field is missing or not a Symbol",
            ));
        };
        self.runtime.primary_provider.graph_ref().with_runtime(execution, |native| {
            use worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial as PinStop;
            if native.runtime_instance_id() != receipt.provider_runtime_instance_id() {
                return Err(invalid(
                    "committed approval receipt belongs to another runtime",
                ));
            }
            let basis = receipt
                .committed_product_publication()
                .retained_relational_basis()
                .ok_or_else(|| {
                    invalid("committed approval has no retained performed poststate")
                })?;
            let observation = basis.observation();
            if observation.commit_receipt() != Some(changes.commit_reference()) {
                return Err(invalid(
                    "committed approval retained poststate differs from its original commit",
                ));
            }
            let view = native
                .read_truth()
                .project_observation(&observation)
                .map_err(|_| {
                    invalid("committed approval retained poststate belongs to another runtime or is unavailable")
                })?;
            let pin = native
                .read_truth()
                .pin_view_read_capability(&view, &[], execution)
                .map_err(|cause| match cause {
                    PinStop::Request(cause) => {
                        Denial::relational_read_stopped(cause, "committed approval symbols")
                    }
                    PinStop::SnapshotUnavailable => {
                        invalid("committed approval symbol snapshot is unavailable")
                    }
                    PinStop::IndexNotInstalled(_) => {
                        invalid("committed approval symbol index is unavailable")
                    }
                })?;
            pin.with_symbol_text(
                execution,
                worth_relational::facade::symbols::Symbol(symbol.0),
                |encoded| {
                    let encoded = encoded.ok_or_else(|| {
                        invalid("committed approval request Symbol is absent from pinned dictionary")
                    })?;
                    if !encoded.is_managed() {
                        return Err(invalid(
                            "committed approval request Symbol lacks native managed custody",
                        ));
                    }
                    let _retained =
                        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                            execution,
                            encoded.as_str(),
                        )
                        .map_err(|cause| match cause {
                            Decode::InvalidRequest => {
                                invalid("committed approval request JSON is invalid")
                            }
                            Decode::Actor(cause) => *cause,
                        })?;
                    Ok(())
                },
            )
            .map_err(|cause| Denial::relational_read_stopped(cause, "committed approval symbols"))?
        })
    }
}
