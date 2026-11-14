//! Explicit request-bound stopped-custody acquisition and cancellation.
use super::super::BranchSetAdoptionResolution;
use super::*;
use crate::domain_computation::primary_graph::query_obligations::{
    QueryAdoptionPerformedDescription, QueryObligationDenial, QueryStoppedBranchSetObligation,
};
use worth_foundational::facade::{CanonicalDigestId, DurableCompositeCommitIdentity};
impl WorthQueryPreparedBranchSetAdoption {
    pub fn begin_resume<Schema: ApplicationSchema>(
        mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryStoppedBranchSetAdoption, (WorthQueryBranchSetAdoptionResumeDenial, Self)>
    {
        let result = application.with_application_advancement(request, |phase| {
            let execution = phase
                .checked_execution_request_for(&application.runtime.product_runtime)
                .map_err(|cause| {
                    WorthQueryBranchSetAdoptionResumeDenial::Preparation(
                        WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                    )
                })?;
            self.stop_in_request(application, &execution)
        });
        match result {
            Ok(Ok(stopped)) => Ok(stopped),
            Ok(Err(cause)) => Err((cause, self)),
            Err(cause) => Err((
                WorthQueryBranchSetAdoptionResumeDenial::Preparation(
                    WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                ),
                self,
            )),
        }
    }
    fn stop_in_request<Schema: ApplicationSchema>(
        &mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryStoppedBranchSetAdoption, WorthQueryBranchSetAdoptionResumeDenial> {
        use WorthQueryBranchSetAdoptionResumeDenial as Denial;
        let (branch, source, attempt) = match self.resolution_required {
            Some(BranchSetAdoptionResolution::NoEffect {
                branch,
                source,
                attempt,
            }) => (branch, source, attempt),
            Some(BranchSetAdoptionResolution::ProductUnpublished(branch)) => {
                let Some(WorthQueryBranchSetAdoptionProgress::ProductUnpublished {
                    adoption, ..
                }) = self.progress.last()
                else {
                    return Err(Denial::NotStopped);
                };
                if !adoption.has_no_owner_effects() {
                    return Err(Denial::NotStopped);
                }
                let actual = adoption.attempt_identity();
                let attempt=actual.owner_identity().durable().map(|owner|worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(owner,actual.ordinal()));
                (branch, *adoption.source(), attempt)
            }
            None => return Err(Denial::NotStopped),
        };
        let support = application
            .installed_program_support()
            .ok_or(Denial::Obligation(
                QueryObligationDenial::SupportUnavailable,
            ))?;
        let selected = application
            .on_branch(branch)
            .select_with_request(execution)
            .map_err(|denial| {
                Denial::Preparation(
                    WorthQueryBranchSetAdoptionPreparationDenial::ProductSelection {
                        branch,
                        denial,
                    },
                )
            })?;
        let binding = selected.product().publication_binding();
        let observation = binding.observation();
        let mut remaining = vec![branch];
        remaining.extend(self.pending.iter().map(|p| p.branch));
        let mut descriptions = vec![(
            observation.branch_identity().name().as_str().to_owned(),
            observation.lifecycle_incarnation().ordinal(),
            CanonicalDigestId::new(*source.as_bytes()),
        )];
        let mut custody =
            vec![support
                .retain_custody(&source, &self.target)
                .ok_or(Denial::Obligation(
                    QueryObligationDenial::SupportUnavailable,
                ))?];
        for pending in &self.pending {
            if pending.adoption.owner_identity
                != application.runtime.product_runtime.owner.owner_identity()
            {
                return Err(Denial::ForeignRuntime);
            }
            let expected = pending.adoption.publication.expected_head();
            descriptions.push((
                expected.branch_identity().name().as_str().to_owned(),
                expected.lifecycle_incarnation().ordinal(),
                CanonicalDigestId::new(*pending.adoption.source().as_bytes()),
            ));
            custody.push(
                support
                    .retain_custody(pending.adoption.source(), &self.target)
                    .ok_or(Denial::Obligation(
                        QueryObligationDenial::SupportUnavailable,
                    ))?,
            );
        }
        if let Some(attempt) = attempt {
            let performed = self.performed_descriptions(application, execution)?;
            let diagnostic = self.stop_diagnostic()?;
            support
                .lifecycle()
                .retain_stopped_adoption(
                    execution,
                    QueryStoppedBranchSetObligation {
                        attempt,
                        target: CanonicalDigestId::new(*self.target.as_bytes()),
                        remaining: descriptions,
                        performed,
                        stop_diagnostic: diagnostic,
                        selection_work: self.total_selection_work_units as u64,
                    },
                )
                .map_err(Denial::Obligation)?;
            for token in &mut custody {
                token.transfer_to_durable();
            }
        }
        Ok(WorthQueryStoppedBranchSetAdoption {
            remaining: remaining.into_boxed_slice(),
            progress: std::mem::take(&mut self.progress),
            target: self.target,
            total_selection_work_units: self.total_selection_work_units,
            owner: application.runtime.product_runtime.owner.owner_identity(),
            durable: attempt,
            _support_custody: custody,
        })
    }
    fn performed_descriptions<Schema: ApplicationSchema>(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<QueryAdoptionPerformedDescription>, WorthQueryBranchSetAdoptionResumeDenial>
    {
        use WorthQueryBranchSetAdoptionResumeDenial as Denial;
        let mut performed = Vec::new();
        for progress in &self.progress {
            match progress {
                WorthQueryBranchSetAdoptionProgress::Performed { branch, adoption } => {
                    let commit = adoption.product_commit();
                    performed.push(QueryAdoptionPerformedDescription {
                        branch: adoption
                            .product_branch_identity()
                            .name()
                            .as_str()
                            .to_owned(),
                        lifecycle: branch.occurrence().ordinal(),
                        source: CanonicalDigestId::new(*adoption.source().as_bytes()),
                        target: CanonicalDigestId::new(*adoption.target().as_bytes()),
                        commit: DurableCompositeCommitIdentity::from_recorded(
                            commit
                                .owner_identity()
                                .durable()
                                .ok_or(Denial::ForeignRuntime)?,
                            commit.ordinal(),
                        ),
                    });
                }
                WorthQueryBranchSetAdoptionProgress::RecordedPerformed {
                    branch,
                    commit,
                    source,
                    target,
                } => {
                    let actual = application
                        .on_branch(*branch)
                        .select_with_request(execution)
                        .map_err(|denial| {
                            Denial::Preparation(
                                WorthQueryBranchSetAdoptionPreparationDenial::ProductSelection {
                                    branch: *branch,
                                    denial,
                                },
                            )
                        })?;
                    performed.push(QueryAdoptionPerformedDescription {
                        branch: actual
                            .product()
                            .publication_binding()
                            .observation()
                            .branch_identity()
                            .name()
                            .as_str()
                            .to_owned(),
                        lifecycle: branch.occurrence().ordinal(),
                        source: *source,
                        target: *target,
                        commit: *commit,
                    });
                }
                _ => {}
            }
        }
        Ok(performed)
    }
    fn stop_diagnostic(&self) -> Result<String, WorthQueryBranchSetAdoptionResumeDenial> {
        use WorthQueryBranchSetAdoptionResumeDenial as Denial;
        let diagnostic =
            match self.progress.last() {
                Some(WorthQueryBranchSetAdoptionProgress::NoEffect { no_effect, .. }) => {
                    format!("{:?}", no_effect.cause())
                }
                Some(WorthQueryBranchSetAdoptionProgress::ExecutionDenied { cause, .. }) => {
                    format!("{cause:?}")
                }
                Some(WorthQueryBranchSetAdoptionProgress::ProductUnpublished {
                    adoption, ..
                }) if adoption.has_no_owner_effects() => format!("{:?}", adoption.cause()),
                _ => return Err(Denial::NotStopped),
            };
        Ok(diagnostic)
    }
}
impl WorthQueryStoppedBranchSetAdoption {
    pub fn cancel<Schema: ApplicationSchema>(
        self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchSetAdoptionCancellation, WorthQueryBranchSetAdoptionResumeFailure>
    {
        let result = application
            .with_application_advancement(request, |phase| {
                if self.owner != application.runtime.product_runtime.owner.owner_identity() {
                    return Err(WorthQueryBranchSetAdoptionResumeDenial::ForeignRuntime);
                }
                let execution = phase
                    .checked_execution_request_for(&application.runtime.product_runtime)
                    .map_err(|cause| {
                        WorthQueryBranchSetAdoptionResumeDenial::Preparation(
                            WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                        )
                    })?;
                if let Some(attempt) = self.durable {
                    application
                        .installed_program_support()
                        .expect("retained support")
                        .lifecycle()
                        .release_stopped_adoption(&execution, attempt)
                        .map_err(WorthQueryBranchSetAdoptionResumeDenial::Obligation)?;
                }
                Ok(())
            })
            .unwrap_or_else(|cause| {
                Err(WorthQueryBranchSetAdoptionResumeDenial::Preparation(
                    WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                ))
            });
        if let Err(denial) = result {
            return Err(resume_failure(denial, self));
        }
        Ok(WorthQueryBranchSetAdoptionCancellation {
            cancelled_branch_count: self.remaining.len(),
            progress: self.progress,
            total_selection_work_units: self.total_selection_work_units,
        })
    }
}
