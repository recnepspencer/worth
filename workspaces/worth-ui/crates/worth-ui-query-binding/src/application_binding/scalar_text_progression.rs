use worth_foundational::facade::InternedString;
use worth_query::facade::{
    foundation::{ConsumedNativeRefinementDenial, ObservationLaneWitness},
    installed::{self, operation},
    read, runtime,
};

use super::{
    WorthUiBoundScalarTextProjection, WorthUiInstalledScalarTextOperationReference,
    WorthUiScalarTextConsumptionOutcome, WorthUiScalarTextExecutionOutcome,
    WorthUiScalarTextNativeAccess, WorthUiScalarTextNativeRequest,
    WorthUiScalarTextNativeRequestDenial, WorthUiScalarTextOperatingWorldGateway,
    WorthUiScalarTextPublicationOutcome, WorthUiScalarTextSettlementOutcome,
};

type Executed = operation::WorthQueryExecutedDomainOperation<
    crate::WorthUiDomainEntry,
    crate::WorthUiScalarTextProjection,
    crate::WorthUiScalarTextProjectionFamily,
    ObservationLaneWitness,
    read::WorthQueryReadCompletion,
>;
type Published = operation::WorthQueryPublishedDomainOperation<
    crate::WorthUiDomainEntry,
    crate::WorthUiScalarTextProjection,
    crate::WorthUiScalarTextProjectionFamily,
    ObservationLaneWitness,
>;
type Consumed = operation::WorthQueryConsumedDomainProjection<
    crate::WorthUiDomainEntry,
    crate::WorthUiScalarTextProjection,
    crate::WorthUiScalarTextProjectionFamily,
    ObservationLaneWitness,
>;
type Settled = operation::WorthQuerySettledDomainProjection<
    crate::WorthUiDomainEntry,
    crate::WorthUiScalarTextProjection,
    crate::WorthUiScalarTextProjectionFamily,
    ObservationLaneWitness,
>;

pub(crate) enum WorthUiScalarTextConsumerPreparationDenial {
    Binding(Box<installed::WorthQueryOperationBindingDenial>),
    ConsumerContract(operation::WorthQueryConsumerProjectionContractDenial),
    NativeRequest(WorthUiScalarTextNativeRequestDenial),
}

pub(crate) struct WorthUiPreparedScalarTextConsumer {
    reference: WorthUiInstalledScalarTextOperationReference,
    bound: WorthUiBoundScalarTextProjection<ObservationLaneWitness>,
    native_request: WorthUiScalarTextNativeRequest,
}

pub(crate) struct WorthUiExecutedScalarTextConsumer {
    reference: WorthUiInstalledScalarTextOperationReference,
    executed: Executed,
    native_request: WorthUiScalarTextNativeRequest,
}

pub(crate) struct WorthUiPublishedScalarTextConsumer {
    reference: WorthUiInstalledScalarTextOperationReference,
    published: Published,
    native_request: WorthUiScalarTextNativeRequest,
}

pub(crate) struct WorthUiConsumedScalarTextProjection {
    reference: WorthUiInstalledScalarTextOperationReference,
    consumed: Consumed,
    native_access: WorthUiScalarTextNativeAccess,
}

pub(crate) struct WorthUiSettledScalarTextProjection {
    reference: WorthUiInstalledScalarTextOperationReference,
    settled: Settled,
    native_access: WorthUiScalarTextNativeAccess,
}

pub(crate) struct WorthUiDerivedScalarTextProjection {
    settled: WorthUiSettledScalarTextProjection,
    value: crate::UiNativeTextValue,
    access_counters: operation::WorthQueryNativeAccessCounters,
}

pub(crate) enum WorthUiScalarTextDerivationStop {
    NativeAccess {
        denial: operation::WorthQueryNativeAccessDenial,
        resolution_counters: worth_query::facade::domain::WorthQueryNativeKeyResolutionCounters,
    },
    NativeRefinement {
        denial: ConsumedNativeRefinementDenial,
        resolution_counters: worth_query::facade::domain::WorthQueryNativeKeyResolutionCounters,
        access_counters: operation::WorthQueryNativeAccessCounters,
    },
    SymbolicText {
        resolution_counters: worth_query::facade::domain::WorthQueryNativeKeyResolutionCounters,
        access_counters: operation::WorthQueryNativeAccessCounters,
    },
    BudgetExceeded {
        byte_len: usize,
        limit: usize,
        resolution_counters: worth_query::facade::domain::WorthQueryNativeKeyResolutionCounters,
        access_counters: operation::WorthQueryNativeAccessCounters,
    },
}

impl WorthUiScalarTextOperatingWorldGateway<'_> {
    #[allow(
        clippy::result_large_err,
        reason = "cold preparation preserves Query's exact proof-carrying denial"
    )]
    pub(crate) fn prepare_consumer(
        self,
        selected_field: &crate::UiProjectionFieldRequirement,
    ) -> Result<WorthUiPreparedScalarTextConsumer, WorthUiScalarTextConsumerPreparationDenial> {
        let (reference, bound) = self
            .bind()
            .map_err(WorthUiScalarTextConsumerPreparationDenial::Binding)?;
        let consumer = bound
            .consumer_projection_contract()
            .map_err(WorthUiScalarTextConsumerPreparationDenial::ConsumerContract)?;
        let native_request =
            WorthUiScalarTextNativeRequest::from_consumer(consumer, selected_field)
                .map_err(WorthUiScalarTextConsumerPreparationDenial::NativeRequest)?;
        Ok(WorthUiPreparedScalarTextConsumer {
            reference,
            bound,
            native_request,
        })
    }
}

impl WorthUiPreparedScalarTextConsumer {
    pub(crate) fn binding_identity_for_reporting(&self) -> &str {
        self.bound.binding_identity()
    }

    #[allow(
        clippy::result_large_err,
        reason = "replacement is a cold authority transition with an exact Query denial"
    )]
    pub(crate) fn replacement_witness_for(
        &self,
        candidate: &Self,
    ) -> Result<
        worth_query::facade::domain::WorthQueryReplacementWitness,
        worth_query::facade::domain::WorthQueryReplacementDenial,
    > {
        self.bound.replacement_with(&candidate.bound)
    }

    pub(crate) fn execute(
        self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> WorthUiScalarTextExecutionOutcome {
        let admitted =
            match installed::transition::resource_admission(self.bound.admit_execution_resources(
                (),
                crate::installed_domain::execution_resources::operation_execution_resource_request(
                ),
                workspace,
            ))
            .into_result()
            {
                Ok(admitted) => admitted,
                Err(stop) => {
                    return WorthUiScalarTextExecutionOutcome::ResourceAdmission(Box::new(stop));
                }
            };
        match installed::transition::execution(admitted.execute(workspace)) {
            installed::transition::WorthQueryExecutionTransition::Executed(executed) => {
                WorthUiScalarTextExecutionOutcome::Executed(Box::new(
                    WorthUiExecutedScalarTextConsumer {
                        reference: self.reference,
                        executed,
                        native_request: self.native_request,
                    },
                ))
            }
            installed::transition::WorthQueryExecutionTransition::Deferred(deferred) => {
                WorthUiScalarTextExecutionOutcome::Deferred(Box::new(deferred))
            }
            installed::transition::WorthQueryExecutionTransition::Denied(stop) => {
                WorthUiScalarTextExecutionOutcome::Denied(Box::new(stop))
            }
            installed::transition::WorthQueryExecutionTransition::Stale(stop) => {
                WorthUiScalarTextExecutionOutcome::Stale(Box::new(stop))
            }
            installed::transition::WorthQueryExecutionTransition::RebindRequired(stop) => {
                WorthUiScalarTextExecutionOutcome::RebindRequired(Box::new(stop))
            }
            installed::transition::WorthQueryExecutionTransition::Failed(stop) => {
                WorthUiScalarTextExecutionOutcome::Failed(Box::new(stop))
            }
        }
    }
}

impl WorthUiExecutedScalarTextConsumer {
    pub(crate) fn publish(self) -> WorthUiScalarTextPublicationOutcome {
        match installed::transition::publication(self.executed.publish()) {
            installed::transition::WorthQueryPublicationTransition::Published(published) => {
                WorthUiScalarTextPublicationOutcome::Published(Box::new(
                    WorthUiPublishedScalarTextConsumer {
                        reference: self.reference,
                        published,
                        native_request: self.native_request,
                    },
                ))
            }
            installed::transition::WorthQueryPublicationTransition::Denied(stop) => {
                WorthUiScalarTextPublicationOutcome::Denied(Box::new(stop))
            }
            installed::transition::WorthQueryPublicationTransition::Stale(stop) => {
                WorthUiScalarTextPublicationOutcome::Stale(Box::new(stop))
            }
            installed::transition::WorthQueryPublicationTransition::RebindRequired(stop) => {
                WorthUiScalarTextPublicationOutcome::RebindRequired(Box::new(stop))
            }
            installed::transition::WorthQueryPublicationTransition::Failed(stop) => {
                WorthUiScalarTextPublicationOutcome::Failed(Box::new(stop))
            }
        }
    }
}

impl WorthUiPublishedScalarTextConsumer {
    pub(crate) fn consume(self) -> WorthUiScalarTextConsumptionOutcome {
        let (request, native_access) = self.native_request.into_parts();
        match installed::transition::consumption(self.published.consume_bound(request)) {
            installed::transition::WorthQueryConsumptionTransition::Consumed(consumed) => {
                WorthUiScalarTextConsumptionOutcome::Consumed(Box::new(
                    WorthUiConsumedScalarTextProjection {
                        reference: self.reference,
                        consumed,
                        native_access,
                    },
                ))
            }
            installed::transition::WorthQueryConsumptionTransition::Denied(stop) => {
                WorthUiScalarTextConsumptionOutcome::Denied(Box::new(stop))
            }
            installed::transition::WorthQueryConsumptionTransition::Deferred(stop) => {
                WorthUiScalarTextConsumptionOutcome::Deferred(Box::new(stop))
            }
            installed::transition::WorthQueryConsumptionTransition::Stale(stop) => {
                WorthUiScalarTextConsumptionOutcome::Stale(Box::new(stop))
            }
            installed::transition::WorthQueryConsumptionTransition::RebindRequired(stop) => {
                WorthUiScalarTextConsumptionOutcome::RebindRequired(Box::new(stop))
            }
            installed::transition::WorthQueryConsumptionTransition::Failed(stop) => {
                WorthUiScalarTextConsumptionOutcome::Failed(Box::new(stop))
            }
        }
    }
}

impl WorthUiConsumedScalarTextProjection {
    pub(crate) fn settle(self) -> WorthUiScalarTextSettlementOutcome {
        match installed::transition::settlement(self.consumed.settle()) {
            installed::transition::WorthQuerySettlementTransition::Settled(settled) => {
                WorthUiScalarTextSettlementOutcome::Settled(Box::new(
                    WorthUiSettledScalarTextProjection {
                        reference: self.reference,
                        settled,
                        native_access: self.native_access,
                    },
                ))
            }
            installed::transition::WorthQuerySettlementTransition::Denied(stop) => {
                WorthUiScalarTextSettlementOutcome::Denied(Box::new(stop))
            }
            installed::transition::WorthQuerySettlementTransition::Stale(stop) => {
                WorthUiScalarTextSettlementOutcome::Stale(Box::new(stop))
            }
            installed::transition::WorthQuerySettlementTransition::RebindRequired(stop) => {
                WorthUiScalarTextSettlementOutcome::RebindRequired(Box::new(stop))
            }
            installed::transition::WorthQuerySettlementTransition::Failed(stop) => {
                WorthUiScalarTextSettlementOutcome::Failed(Box::new(stop))
            }
        }
    }
}

impl WorthUiSettledScalarTextProjection {
    pub(crate) fn derive_native_text(
        self,
        budget: crate::UiProjectionConsumptionBudget,
    ) -> Result<WorthUiDerivedScalarTextProjection, Box<WorthUiScalarTextDerivationStop>> {
        let resolution_counters = self.native_access.resolution_counters();
        let access = match self.settled.native_value(self.native_access.key(), 0) {
            Ok(access) => access,
            Err(denial) => {
                return Err(Box::new(WorthUiScalarTextDerivationStop::NativeAccess {
                    denial,
                    resolution_counters,
                }));
            }
        };
        let access_counters = access.counters();
        let text = match access.fact().as_interned_string() {
            Ok(InternedString::Raw(text)) => text.clone(),
            Ok(InternedString::Stored(text)) => text.as_str().to_owned(),
            Ok(InternedString::Symbol(_)) => {
                return Err(Box::new(WorthUiScalarTextDerivationStop::SymbolicText {
                    resolution_counters,
                    access_counters,
                }));
            }
            Err(denial) => {
                return Err(Box::new(
                    WorthUiScalarTextDerivationStop::NativeRefinement {
                        denial,
                        resolution_counters,
                        access_counters,
                    },
                ));
            }
        };
        let byte_len = text.len();
        if byte_len > budget.native_bytes_retained() {
            return Err(Box::new(WorthUiScalarTextDerivationStop::BudgetExceeded {
                byte_len,
                limit: budget.native_bytes_retained(),
                resolution_counters,
                access_counters,
            }));
        }
        Ok(WorthUiDerivedScalarTextProjection {
            settled: self,
            value: crate::UiNativeTextValue::from_raw(text),
            access_counters,
        })
    }

    #[cfg(any(test, feature = "certification-construction"))]
    pub(crate) fn certification_native_key(&self) -> &operation::WorthQueryNativeAccessKey {
        self.native_access.key()
    }

    #[cfg(any(test, feature = "certification-construction"))]
    pub(crate) fn certification_contract_digest(&self) -> String {
        crate::reporting_projection::scalar_contract_digest_for_reporting(self)
    }

    #[cfg(any(test, feature = "certification-construction"))]
    pub(crate) fn certification_settled_projection(&self) -> &Settled {
        &self.settled
    }

    #[cfg(any(test, feature = "certification-construction"))]
    #[allow(
        clippy::result_large_err,
        reason = "certification exposes the exact Query denial under fault injection"
    )]
    pub(crate) fn certification_native_value<'a>(
        &'a self,
        key: &operation::WorthQueryNativeAccessKey,
    ) -> Result<operation::WorthQueryNativeFieldAccess<'a>, operation::WorthQueryNativeAccessDenial>
    {
        self.settled.native_value(key, 0)
    }
}

impl WorthUiDerivedScalarTextProjection {
    pub(crate) fn into_value(self) -> crate::UiNativeTextValue {
        self.value
    }

    pub(crate) fn access_counters(&self) -> operation::WorthQueryNativeAccessCounters {
        self.access_counters
    }

    pub(crate) fn resolution_counters(
        &self,
    ) -> worth_query::facade::domain::WorthQueryNativeKeyResolutionCounters {
        self.settled.native_access.resolution_counters()
    }

    pub(crate) fn installation_is_current(&self) -> bool {
        self.settled.reference.installation_is_current()
    }
}
