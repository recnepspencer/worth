//! Original recovered delivery rejoins the ordinary Bridge/readiness engines.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking;
use crate::domain_computation::primary_graph::WorthQueryRecoveredProducerPublication;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn advance_recovered_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        producer: &str,
        checkpoint: Checkpoint,
        admitted: Option<&WorthQueryRequestScope>,
        finish: impl FnOnce(
            Checkpoint,
            Option<WorthQueryOutputDemandDenial>,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::RecoveredPublished {
                publication,
                delivery,
                ready_backing,
            } => {
                let prepared = match admitted {
                    None => None,
                    Some(request) => {
                        let prepared =
                            match self.prepare_selected_delivery(producer, publication.view()) {
                                Ok(prepared) => prepared,
                                Err(stop) => {
                                    finish(
                                        Checkpoint::RecoveredPublished {
                                            publication,
                                            delivery,
                                            ready_backing,
                                        },
                                        Some(stop),
                                    )?;
                                    return Ok(false);
                                }
                            };
                        if let Err(stop) = selected::check_request(request) {
                            finish(
                                Checkpoint::RecoveredPublished {
                                    publication,
                                    delivery,
                                    ready_backing,
                                },
                                Some(stop),
                            )?;
                            return Ok(false);
                        }
                        Some(prepared)
                    }
                };
                own_effect_observation::record(Stage::DeliveryStarted);
                let mut delivered = false;
                self.deliver_output_checkpoint_with_finish(
                    phase,
                    producer,
                    publication::Publication::Recovered(publication),
                    delivery,
                    ready_backing,
                    admitted.is_some(),
                    prepared,
                    |checkpoint, stop| {
                        delivered = matches!(&checkpoint, Checkpoint::RecoveredDelivered { .. })
                            && stop.is_none();
                        let stop = stop.or_else(|| {
                            admitted.and_then(|request| selected::check_request(request).err())
                        });
                        finish(checkpoint, stop)
                    },
                )?;
                own_effect_observation::record(if delivered {
                    Stage::Delivered
                } else {
                    Stage::DeliveryWait
                });
                Ok(delivered)
            }
            Checkpoint::RecoveredDelivered {
                publication,
                delivery,
                ready_backing,
            } => self.finish_recovered_readiness(
                execution,
                phase,
                producer,
                publication,
                delivery,
                ready_backing,
                admitted,
                finish,
            ),
            Checkpoint::PublicationPending { .. }
            | Checkpoint::Published { .. }
            | Checkpoint::Delivered { .. }
            | Checkpoint::Ready(_) => {
                unreachable!("recovered checkpoint dispatch retains its phase")
            }
        }
    }
    fn finish_recovered_readiness(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        producer: &str,
        publication: WorthQueryRecoveredProducerPublication,
        delivery: Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready_backing: PreparedReadyBacking,
        admitted: Option<&WorthQueryRequestScope>,
        finish: impl FnOnce(
            Checkpoint,
            Option<WorthQueryOutputDemandDenial>,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let readiness = match admitted {
            None => self.evaluate_current_output_readiness(
                execution,
                phase,
                producer,
                publication.view(),
                delivery.as_ref(),
                &ready_backing,
            ),
            Some(request) => self.evaluate_current_output_readiness_admitted(
                execution,
                phase,
                producer,
                publication.view(),
                delivery.as_ref(),
                &ready_backing,
                request,
            ),
        };
        let readiness = match readiness {
            Ok(readiness) => readiness,
            Err(stop) => {
                finish(
                    Checkpoint::RecoveredDelivered {
                        publication,
                        delivery,
                        ready_backing,
                    },
                    Some(stop),
                )?;
                return Ok(false);
            }
        };
        own_effect_observation::record(Stage::Ready);
        let completed = ready_backing.complete(WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Recovered(publication),
            readiness,
        });
        finish(Checkpoint::Ready(completed), None)?;
        Ok(true)
    }
}
