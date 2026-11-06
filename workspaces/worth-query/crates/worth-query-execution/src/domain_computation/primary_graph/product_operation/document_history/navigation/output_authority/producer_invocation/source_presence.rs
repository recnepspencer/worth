//! Historical identity evidence belongs to this navigation's original D.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAdmissionDenial as Denial,
    WorthQueryApplicationQueryAdmissionDenialKind as Kind,
};

impl HistoryProducerInvocation<'_> {
    pub(in crate::domain_computation::primary_graph) fn family(&self) -> &'static str {
        self.family
    }

    pub(in crate::domain_computation::primary_graph) fn historical_branch(
        &self,
    ) -> Option<crate::basis::WorthQueryProductBranch> {
        self.restored_basis.selected().map(|_| {
            crate::basis::WorthQueryProductBranch::from_occurrence(
                self.restored_basis
                    .observation()
                    .product_observation()
                    .lifecycle_incarnation(),
            )
        })
    }

    pub(in crate::domain_computation) fn admits_historical_truth<S: ApplicationSchema>(
        &self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            S,
        >,
    ) -> bool {
        self.restored_basis.selected().is_some()
            && selected.product().observation().lifecycle_incarnation()
                == self
                    .restored_basis
                    .observation()
                    .product_observation()
                    .lifecycle_incarnation()
    }

    /// The permission owner calls this only after C has authorized this exact
    /// source. No identity failure at D can precede that authorization.
    pub(in crate::domain_computation) fn require_historical_source<S, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        scope: &WorthQueryApplicationEntityIdentity<S, E>,
        subject: &str,
    ) -> Result<(), Denial> {
        use worth_relational::facade::runtime::{
            RelationalHistoricalSelectionReadDenial as Stop,
            RelationalHistoricalSourcePresence as Presence,
        };
        let answer = self
            .restored_basis
            .native(execution, native)
            .and_then(|evidence| {
                evidence.source_presence(execution, scope.entity_id(), scope.entity_kind())
            });
        match answer {
            Ok(Presence::LiveInDomain) => Ok(()),
            Ok(Presence::NotLive) => Err(Denial::new(
                Kind::HistoricalSourceNotLive {
                    source: scope.entity_id(),
                },
                subject,
            )),
            Ok(Presence::KindMismatch { actual }) => Err(Denial::new(
                Kind::HistoricalSourceKindMismatch {
                    source: scope.entity_id(),
                    actual,
                },
                subject,
            )),
            Ok(Presence::OutsideDomain) => Err(Denial::new(
                Kind::HistoricalSourceOutsideDomain {
                    source: scope.entity_id(),
                },
                subject,
            )),
            Err(
                Stop::ForeignOwner
                | Stop::NotGraphSelection { .. }
                | Stop::SelectedBasisMismatch { .. },
            ) => Err(Denial::new(Kind::StaleScope, subject)),
            Err(Stop::Execution(cause)) => Err(Denial::new(execution_kind(cause), subject)),
            Err(Stop::Request(cause)) => Err(Denial::new(execution_kind(cause.into()), subject)),
        }
    }
}

fn execution_kind(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Kind {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None) {
        Ok(kind) => Kind::ExecutionRequest(kind),
        Err(Stop::Cancelled) => Kind::Cancelled,
        Err(Stop::TimedOut) => Kind::DeadlineExceeded,
    }
}
