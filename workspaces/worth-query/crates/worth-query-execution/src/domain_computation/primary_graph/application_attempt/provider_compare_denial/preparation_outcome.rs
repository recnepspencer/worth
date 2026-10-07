//! Observe real preparation refusals through the application denial boundary.
use super::super::WorthQueryApplicationCommitOutcome as Outcome;
use crate::domain_computation::{
    WorthQueryProviderCompareAndCommitDenial as Compare,
    WorthQueryProviderSessionCommitStop as Stop,
};

pub(in crate::domain_computation::primary_graph) fn application_outcome(stop: Stop) -> Outcome {
    match stop {
        Stop::PreEffectDenied(failure) => {
            let super::Progression::Denied(denial) =
                super::provider_compare_denied(Compare::ProviderSession(failure))
            else {
                panic!("pre-effect refusal must remain an application denial");
            };
            Outcome::Denied(denial)
        }
        Stop::ControlStopped(stopped) => match super::control_stopped_outcome(stopped) {
            super::Progression::Denied(denial) => Outcome::Denied(denial),
            super::Progression::Cancelled => Outcome::Cancelled,
            super::Progression::TimedOut => Outcome::TimedOut,
            _ => panic!("wrong control disposition"),
        },
        Stop::Denied(_) => panic!("preparation refusal incorrectly requests commit recovery"),
        Stop::Deferred(_)
        | Stop::SettlementDeferred(_)
        | Stop::ProductStale(_)
        | Stop::ProductUnpublished(_)
        | Stop::NoEffect(_) => panic!("wrong preparation disposition"),
    }
}
