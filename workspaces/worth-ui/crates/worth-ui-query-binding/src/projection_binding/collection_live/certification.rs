use worth_foundational::facade::{AspectValue, InternedString};
use worth_query::facade::{
    installed::{collection, observation, operation},
    runtime,
};

use super::UiLiveCollectionProjection;

type QueryPatch = collection::WorthQueryCollectionPatch;
type QueryReceipt = collection::WorthQueryCollectionPatchApplicationReceipt;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CollectionProjectionStateSnapshot {
    rows: Vec<(runtime::WorthQueryEvidenceIdentityKey, Box<[String]>)>,
    result_state: operation::WorthQueryOperationResultState,
    continuation: Option<runtime::WorthQueryEvidenceIdentityKey>,
    source_generation: runtime::WorthQueryEvidenceIdentityKey,
    result_generation: runtime::WorthQueryEvidenceIdentityKey,
    warnings: Box<[collection::WorthQueryCollectionWindowWarning]>,
    reset_pending: bool,
}

impl UiLiveCollectionProjection {
    pub(crate) fn certification_plan_patch_twins(
        &mut self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> (QueryPatch, QueryPatch) {
        workspace
            .with_execution_request_mut(|request_execution, workspace| {
                let delivery = self
                    .lease
                    .drain(workspace)
                    .expect("certification collection delivery drains");
                let delta = self
                    .lease
                    .consumer_invalidation_delta(delivery)
                    .expect("certification collection delta derives");
                let admitted = self
                    .lease
                    .admit_consumer_invalidation_delta(delta, workspace)
                    .unwrap_or_else(|stop| {
                        panic!(
                            "certification collection invalidation stopped: {:?}",
                            stop.kind()
                        )
                    });
                self.consumer
                    .bind_shared_target(&admitted, workspace)
                    .expect("certification consumer binds");
                (
                    required_patch(request_execution, &mut self.consumer, &admitted, workspace),
                    required_patch(request_execution, &mut self.consumer, &admitted, workspace),
                )
            })
            .expect("fixture workspace host call opens")
    }

    pub(crate) fn certification_plan_patch_for_target(
        &mut self,
        target: &mut Self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> (QueryPatch, QueryPatch, QueryPatch) {
        workspace
            .with_execution_request_mut(|request_execution, workspace| {
                let delivery = self
                    .lease
                    .drain(workspace)
                    .expect("certification shared delivery drains");
                let delta = self
                    .lease
                    .consumer_invalidation_delta(delivery)
                    .expect("certification shared delta derives");
                let admitted = self
                    .lease
                    .admit_consumer_invalidation_delta(delta, workspace)
                    .unwrap_or_else(|stop| {
                        panic!(
                            "certification shared invalidation stopped: {:?}",
                            stop.kind()
                        )
                    });
                self.consumer
                    .bind_shared_target(&admitted, workspace)
                    .expect("source consumer binds shared target");
                target
                    .consumer
                    .bind_shared_target(&admitted, workspace)
                    .expect("target consumer binds shared target");
                (
                    required_patch(request_execution, &mut self.consumer, &admitted, workspace),
                    required_patch(request_execution, &mut self.consumer, &admitted, workspace),
                    required_patch(
                        request_execution,
                        &mut target.consumer,
                        &admitted,
                        workspace,
                    ),
                )
            })
            .expect("fixture workspace host call opens")
    }

    #[allow(
        clippy::result_large_err,
        reason = "certification fault injection preserves the exact Query patch denial"
    )]
    pub(crate) fn certification_apply_patch(
        &mut self,
        patch: QueryPatch,
    ) -> Result<QueryReceipt, collection::WorthQueryCollectionDeliveryDenial> {
        self.consumer.apply_patch(patch)
    }

    pub(crate) fn certification_derive_fact(
        &self,
        receipt: &QueryReceipt,
    ) -> crate::UiCollectionProjectionFactReceipt {
        crate::projection_consumption::derive_applied_collection_projection(
            crate::projection_consumption::UiCollectionDerivationContext {
                binding: &self.binding,
                consumer: &self.consumer,
                text_accesses: &self.text_accesses,
                application_item_key_access: self.application_item_key_access.as_ref(),
                budget: self.budget,
            },
            receipt,
        )
    }

    pub(crate) fn certification_state_snapshot(&self) -> CollectionProjectionStateSnapshot {
        let rows =
            self.consumer
                .rows()
                .iter()
                .map(|row| {
                    let values =
                        self.text_accesses
                            .iter()
                            .map(|access| {
                                let fact = self.consumer.native_value(row, access.key()).expect(
                                    "certification state snapshot uses admitted native access",
                                );
                                match fact.native_value().scalar() {
                                    Some(AspectValue::String(InternedString::Raw(value))) => {
                                        value.to_string()
                                    }
                                    other => panic!(
                                        "certification collection value is not raw text: {other:?}"
                                    ),
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice();
                    (
                        row.entity_identity().evidence_identity().operational_key(),
                        values,
                    )
                })
                .collect();
        CollectionProjectionStateSnapshot {
            rows,
            result_state: self.consumer.result_state(),
            continuation: self
                .consumer
                .continuation()
                .identity_evidence()
                .map(|identity| identity.operational_key()),
            source_generation: self
                .consumer
                .source_generation_identity_evidence()
                .operational_key(),
            result_generation: self
                .consumer
                .result_generation_identity_evidence()
                .operational_key(),
            warnings: self.consumer.warnings().into(),
            reset_pending: self.consumer.reset_pending(),
        }
    }
}

fn required_patch(
    request_execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    consumer: &mut collection::WorthQueryCollectionConsumerWindow,
    admitted: &observation::WorthQueryAdmittedConsumerInvalidation<'_>,
    workspace: &runtime::WorthQueryWorkspace,
) -> QueryPatch {
    match consumer.plan_patch(request_execution, admitted, workspace) {
        collection::WorthQueryCollectionDeliveryOutcome::Patch(patch) => patch,
        collection::WorthQueryCollectionDeliveryOutcome::NoDelivery(stop) => {
            panic!(
                "certification semantic mutation did not produce a patch: {:?}",
                stop.kind()
            )
        }
    }
}
