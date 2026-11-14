//! Stages retain exact Native positions and their nominal Query sessions.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::authored_edit::{
    AuthoredEditStageContext, CapturedAuthoredEditStage,
};
impl<'a, 'run, 'phase, S: ApplicationSchema> CoupledApplicationAssembly<'a, 'run, 'phase, S> {
    pub(super) fn stage_captures(&self) -> &[CapturedAuthoredEditStage<S>] {
        self.all_stage_captures().get(1..).unwrap_or_default()
    }
    pub(super) fn all_stage_captures(&self) -> &[CapturedAuthoredEditStage<S>] {
        self.demands
            .authored_stages
            .as_ref()
            .map(|stages| {
                stages
                    .downcast_ref::<Vec<CapturedAuthoredEditStage<S>>>()
                    .expect("same-schema authored stage capture")
                    .as_slice()
            })
            .unwrap_or_default()
    }
    pub(super) fn require_authored_basis(&self) -> Result<(), CoupledSourceReadStop> {
        if let Some(stage) = self
            .all_stage_captures()
            .iter()
            .find(|stage| &stage.before != self.before.observation())
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
                    ordinal: stage.identity.ordinal(),
                    binding: stage.binding,
                },
                "the envelope's admitted C basis changed before assembly",
            )
            .into());
        }
        Ok(())
    }
    pub(super) fn prepare_stage(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before_pin: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        physical: usize,
    ) -> Result<DeclaredCoupledOutputDemands, CoupledSourceReadStop> {
        let context = AuthoredEditStageContext {
            phase: self.phase,
            application: self.application,
        };
        let capture = &self.stage_captures()[physical];
        let pin = candidate.pin_source(execution)?;
        let prepared = capture.executor.prepare(
            execution,
            &context,
            self.coordination,
            self.before,
            before_pin,
            &pin,
        )?;
        let crate::domain_computation::primary_graph::application_attempt::InspectedAuthoredApplication { writer, batch, content, roles, demands } = prepared;
        // Retain custody before append: a cold-symbol retry must abort this
        // exact real session even when Native rejects the current suffix.
        let position = self.applied_position() + 1;
        self.authored.borrow_mut().push(AuthoredWriter {
            capture: capture.clone(),
            physical,
            position,
            stage: Some(CoupledWriterStage::Inspected(writer)),
        });
        if let Err(cause) = candidate.append_authored_stage_outputs(
            execution,
            capture.source,
            &content,
            vec![batch],
            roles,
        ) {
            if let Some((target, earlier, later)) = cause.authored_stage_conflict() {
                let ordinal = |position: usize| {
                    if position == 0 {
                        self.demands
                            .authored_stages
                            .as_ref()
                            .and_then(|stages| {
                                stages.downcast_ref::<Vec<CapturedAuthoredEditStage<S>>>()
                            })
                            .and_then(|stages| stages.first())
                            .expect("authored prefix identity")
                            .identity
                            .ordinal()
                    } else {
                        self.stage_captures()[position - 1].identity.ordinal()
                    }
                };
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ConflictingIntent {
                        target: target.clone(),
                        earlier: ordinal(earlier),
                        later: ordinal(later),
                    },
                    format!(
                        "authored stages {} and {} write {target:?}",
                        ordinal(earlier),
                        ordinal(later)
                    ),
                )
                .into());
            }
            return Err(cause.into());
        }
        let after_own = candidate.pin_source(execution)?;
        let writers = self.authored.borrow();
        let Some(CoupledWriterStage::Inspected(writer)) =
            writers.last().and_then(|writer| writer.stage.as_ref())
        else {
            unreachable!("retained authored session")
        };
        self.application
            .runtime
            .primary_provider
            .retain_authored_stage_after_own(
                writer.session(),
                capture.identity,
                capture.binding,
                after_own,
            );
        Ok(demands)
    }
    pub(super) fn admit_authored(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        receipts: &crate::domain_computation::primary_graph::provider::ClosedCoupledInvariantReceipts<'_>,
    ) -> Result<(), crate::domain_computation::WorthQueryInvariantExecutionFailure> {
        let context = AuthoredEditStageContext {
            phase: self.phase,
            application: self.application,
        };
        for writer in self.authored.borrow_mut().iter_mut() {
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.as_ref() else {
                unreachable!("Native inspected the authored stage")
            };
            provider.admit_authored_stage_candidate(
                execution,
                inspected.session(),
                receipts,
                writer.physical,
            )?;
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.take() else {
                unreachable!("retained authored stage")
            };
            match writer
                .capture
                .executor
                .prepare_commit(inspected, execution, &context)
            {
                Ok(prepared) => writer.stage = Some(CoupledWriterStage::Prepared(prepared)),
                Err(outcome) => {
                    let detail = format!(
                        "authored stage {} {} preparation: {outcome:?}",
                        writer.capture.identity.ordinal(),
                        writer.capture.binding
                    );
                    self.denial.borrow_mut().get_or_insert_with(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
                                ordinal: writer.capture.identity.ordinal(),
                                binding: writer.capture.binding,
                            },
                            detail.clone(),
                        )
                        .with_uncommitted(
                            outcome.landed().expect_err("uncommitted stage preparation"),
                        )
                    });
                    return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::new(crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected, detail));
                }
            }
        }
        Ok(())
    }
    pub(super) fn settle_authored(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted> {
        let context = AuthoredEditStageContext {
            phase: self.phase,
            application: self.application,
        };
        let mut failure: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted,
        > = None;
        for writer in self.authored.borrow_mut().drain(..) {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage else {
                unreachable!("validated authored stage")
            };
            let effected = unpublished.is_some() || prepared.has_original_completion();
            if !effected {
                self.retain_abort_outcome(prepared.abort(execution, self.phase, self.application));
                continue;
            }
            if let Err(cause) = writer
                .capture
                .executor
                .complete(prepared, execution, &context, unpublished)
                .landed()
            {
                match (failure.take(), cause) {
                    (Some(prior), crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted::CleanupRequired(cleanup)) => {
                        let (_, cleanup) = cleanup.into_parts();
                        failure = Some(crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted::CleanupRequired(crate::domain_computation::primary_graph::WorthQueryApplicationCleanupRequired::from_parts(prior.into(), cleanup)));
                    }
                    (Some(prior), _) => failure = Some(prior),
                    (None, cause) => failure = Some(cause),
                }
            }
        }
        failure
    }
}
