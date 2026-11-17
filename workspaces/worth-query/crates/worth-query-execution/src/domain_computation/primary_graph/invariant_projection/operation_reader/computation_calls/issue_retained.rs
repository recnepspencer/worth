//! Comparator reads issue current bodies without changing owner-call attribution.
use super::super::decision_reads::structural_walks::WalkRecordDenial;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as Kind;
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget as Target;
use worth_relational::facade::runtime::{
    PinnedAggregateReadDenialKind, PinnedStructuralWalkDenialKind,
    RelationalPinnedReadAdmissionDenial,
};

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// O(body size) fresh Native observation and issuance for structural facts;
    /// other facts retain the ordinary comparator observation path.
    pub(in crate::domain_computation::primary_graph) fn issue_retained(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        comparator: &Comparator,
        key: &WorthQueryApplicationFactKey,
        retained: &Fact,
    ) -> Result<ObservedRetained, WorthQueryApplicationAttemptDenial> {
        let subject = match retained {
            Fact::StructuralWalk { observation } => observation.key().relation(),
            Fact::Aggregate { observation } => observation.key().subject(),
            _ => return self.observe_retained(execution, comparator, key),
        };
        self.admit_retained_relation(subject)?;
        let admission = self
            .admission_identity
            .ok_or_else(|| unavailable(subject))?;
        let contracts = self
            .admitted_contracts
            .clone()
            .ok_or_else(|| unavailable(subject))?;
        let cookie = self.reader.authority_identity;
        let pin = self
            .reader
            .native
            .pin(self.reader.snapshot, execution)
            .map_err(|cause| match cause {
                RelationalPinnedReadAdmissionDenial::Request(cause) => {
                    WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, subject)
                }
                _ => unavailable(subject),
            })?;
        let origin = self.reader.native.origin(self.reader.snapshot);
        let result = execution.run_native_with_checkpoint(|context| {
            let fresh = match retained {
                Fact::StructuralWalk { observation } => {
                    let body = observation
                        .refresh_on_pin_at_roots(
                            &pin,
                            origin,
                            execution,
                            observation.key().roots(),
                            context,
                        )
                        .map_err(|cause| match cause.kind {
                            PinnedStructuralWalkDenialKind::Read(cause) => {
                                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                                    cause, subject,
                                )
                            }
                            _ => unavailable(subject),
                        })?;
                    self.decision_facts
                        .issue_walk_body(&body, admission, contracts, cookie, execution, context)
                        .map_err(|cause| issuance_denial(cause, subject))?;
                    Fact::StructuralWalk { observation: body }
                }
                Fact::Aggregate { observation } => {
                    let body = observation
                        .refresh_on_pin_at_member(
                            &pin,
                            origin,
                            execution,
                            observation.key().member(),
                        )
                        .map_err(|cause| match cause.kind {
                            PinnedAggregateReadDenialKind::Read(cause) => {
                                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                                    cause, subject,
                                )
                            }
                            _ => unavailable(subject),
                        })?;
                    self.decision_facts
                        .issue_aggregate_body(&body, admission, contracts, cookie, context)
                        .map_err(|cause| issuance_denial(cause, subject))?;
                    Fact::Aggregate { observation: body }
                }
                _ => unreachable!("ordinary facts use observe_retained"),
            };
            Ok(ObservedRetained::new(comparator, fresh))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(WorthQueryApplicationAttemptDenial::relational_read_stopped(
                stop.into(),
                subject,
            )),
        }
    }

    fn admit_retained_relation(
        &self,
        relation: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let layout = self
            .reader
            .layout
            .relation(relation)
            .ok_or_else(|| unavailable(relation))?;
        let from = self
            .reader
            .layout
            .entity_name(layout.from)
            .ok_or_else(|| unavailable(relation))?;
        let to = self
            .reader
            .layout
            .entity_name(layout.to)
            .ok_or_else(|| unavailable(relation))?;
        super::super::decision_plan::native_decision::admit_target(
            self.admitted_contracts.as_deref(),
            &Target::Relation {
                relation: relation.to_owned(),
                from: from.to_owned(),
                to: to.to_owned(),
            },
        )
        .map_err(|cause| {
            WorthQueryApplicationAttemptDenial::new(Kind::UndeclaredDecisionRead, cause.subject())
        })
    }
}
fn unavailable(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(Kind::MissingAuthoritativeFact, subject)
}
fn issuance_denial(cause: WalkRecordDenial, subject: &str) -> WorthQueryApplicationAttemptDenial {
    match cause {
        WalkRecordDenial::Execution(cause) => {
            WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, subject)
        }
        WalkRecordDenial::Conflict => {
            WorthQueryApplicationAttemptDenial::new(Kind::ProjectionAdmissionMismatch, subject)
        }
    }
}
