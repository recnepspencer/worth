//! Historical identity evidence belongs to the exact private navigation candidate.
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
        truth: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        subject: &str,
    ) -> Result<(), Denial> {
        use worth_relational::facade::runtime::{
            RelationalHistoricalSelectionReadDenial as Selection,
            RelationalHistoricalSourcePresence as Presence,
            RelationalHistoricalSourceReadDenial as Stop,
        };
        let answer = self
            .restored_basis
            .native(execution, native)
            .map_err(Stop::Selection)
            .and_then(|evidence| {
                if truth.descriptor()
                    == self
                        .restored_basis
                        .observation()
                        .product_observation()
                        .basis()
                        .relational_basis()
                        .descriptor()
                {
                    evidence
                        .source_presence(execution, scope.entity_id(), scope.entity_kind())
                        .map_err(Stop::Selection)
                } else {
                    evidence.source_presence_at(
                        execution,
                        native,
                        truth,
                        runtime
                            .runtime
                            .primary_provider
                            .graph
                            .layout
                            .installed_history_ownership(),
                        scope.entity_id(),
                        scope.entity_kind(),
                    )
                }
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
            Err(Stop::Selection(
                Selection::ForeignOwner
                | Selection::NotGraphSelection { .. }
                | Selection::SelectedBasisMismatch { .. },
            )) => Err(Denial::new(Kind::StaleScope, subject)),
            Err(Stop::Ownership(cause)) => {
                use worth_relational::facade::branch::RelationalModelOwnershipDenial as Owner;
                let kind = match *cause {
                    Owner::Request(cause) => execution_kind(cause.into()),
                    Owner::Native(cause) => execution_kind(cause),
                    Owner::Custody(
                        worth_relational::facade::transactions::TransactionCommitError::Execution {
                            denial,
                            ..
                        },
                    ) => {
                        let worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) = denial.kind;
                        execution_kind(cause)
                    }
                    other => {
                        return Err(Denial::new(
                            Kind::HistoricalSourceOwnershipUnavailable {
                                source: scope.entity_id(),
                            },
                            format!("{subject}: {other:?}"),
                        ))
                    }
                };
                Err(Denial::new(kind, subject))
            }
            Err(Stop::Selection(Selection::Execution(cause))) => {
                Err(Denial::new(execution_kind(cause), subject))
            }
            Err(Stop::Selection(Selection::Request(cause))) => {
                Err(Denial::new(execution_kind(cause.into()), subject))
            }
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
