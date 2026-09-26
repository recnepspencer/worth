use std::sync::Arc;

use crate::capability::UiIntent;

use super::material::{outcome_material, partial_effect};
use super::{UiManagedIntentRecovery, UiManagedIntentRecoveryPoll};
use crate::runtime::intent_execution::{
    UiIntentExecutionAttempt, UiIntentExecutionPollContext, UiIntentExecutionProvider,
    UiIntentExecutionRecovery, UiIntentProviderPoll, UiIntentProviderRecoveryPoll,
    UiIntentProviderSettlement, UiIntentProviderStop,
};

struct UiTypedManagedIntentRecovery<I, Provider>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    recovery: Box<dyn UiIntentExecutionRecovery<I>>,
    _predecessor_provider: Arc<Provider>,
}

struct UiTypedAttemptProtocolRecovery<I, Provider>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    attempt: Box<dyn UiIntentExecutionAttempt<I>>,
    provider: Arc<Provider>,
}

pub(super) fn provider_recovery<I, Provider>(
    recovery: Box<dyn UiIntentExecutionRecovery<I>>,
    provider: Arc<Provider>,
) -> Box<dyn UiManagedIntentRecovery>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    Box::new(UiTypedManagedIntentRecovery::<I, Provider> {
        recovery,
        _predecessor_provider: provider,
    })
}

pub(super) fn protocol_recovery<I, Provider>(
    attempt: Box<dyn UiIntentExecutionAttempt<I>>,
    provider: Arc<Provider>,
) -> Box<dyn UiManagedIntentRecovery>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    Box::new(UiTypedAttemptProtocolRecovery::<I, Provider> { attempt, provider })
}

impl<I, Provider> UiManagedIntentRecovery for UiTypedManagedIntentRecovery<I, Provider>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    fn poll(
        mut self: Box<Self>,
        context: UiIntentExecutionPollContext,
    ) -> UiManagedIntentRecoveryPoll {
        let poll = self.recovery.poll_recovery(context);
        match poll {
            UiIntentProviderRecoveryPoll::Pending => UiManagedIntentRecoveryPoll::Pending(self),
            UiIntentProviderRecoveryPoll::Completed(outcome) => {
                UiManagedIntentRecoveryPoll::Completed(outcome_material(outcome))
            }
            UiIntentProviderRecoveryPoll::Partial(effect) => UiManagedIntentRecoveryPoll::Partial {
                effect: partial_effect(effect),
                recovery: self,
            },
            UiIntentProviderRecoveryPoll::Indeterminate(detail) => {
                UiManagedIntentRecoveryPoll::Indeterminate {
                    detail,
                    recovery: self,
                }
            }
            UiIntentProviderRecoveryPoll::Failed(detail) => UiManagedIntentRecoveryPoll::Failed {
                detail,
                recovery: self,
            },
        }
    }
}

impl<I, Provider> UiManagedIntentRecovery for UiTypedAttemptProtocolRecovery<I, Provider>
where
    I: UiIntent,
    Provider: UiIntentExecutionProvider<I>,
{
    fn poll(
        mut self: Box<Self>,
        context: UiIntentExecutionPollContext,
    ) -> UiManagedIntentRecoveryPoll {
        let poll = self.attempt.poll(context);
        match poll {
            UiIntentProviderPoll::Settled(UiIntentProviderSettlement::Completed(outcome)) => {
                UiManagedIntentRecoveryPoll::Completed(outcome_material(outcome))
            }
            UiIntentProviderPoll::Settled(UiIntentProviderSettlement::Partial(
                effect,
                recovery,
            )) => UiManagedIntentRecoveryPoll::Partial {
                effect: partial_effect(effect),
                recovery: provider_recovery(recovery, self.provider),
            },
            UiIntentProviderPoll::Settled(UiIntentProviderSettlement::Indeterminate(recovery)) => {
                UiManagedIntentRecoveryPoll::Indeterminate {
                    detail: UiIntentProviderStop::stable(
                        "worth_ui.provider.protocol_recovery_indeterminate",
                    ),
                    recovery: provider_recovery(recovery, self.provider),
                }
            }
            UiIntentProviderPoll::PendingBeforeEffect
            | UiIntentProviderPoll::PendingEffectMayHaveBegun
            | UiIntentProviderPoll::Settled(_) => UiManagedIntentRecoveryPoll::Indeterminate {
                detail: UiIntentProviderStop::stable(
                    "worth_ui.provider.protocol_recovery_unresolved",
                ),
                recovery: self,
            },
        }
    }
}
