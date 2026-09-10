use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn admit_pending(
        &mut self,
        correspondence: WorthUiPresentationRuntimeCorrespondence,
    ) -> Result<WorthUiPresentationPendingReceipt, WorthUiPresentationPendingAdmissionDenial> {
        let (authority, _correspondence_nonce, basis) = correspondence.into_parts();
        if !correspondence::is_correspondence_authority(&self.correspondence_authority, &authority)
        {
            return Err(WorthUiPresentationPendingAdmissionDenial::ForeignCorrespondenceAuthority);
        }
        let key = PresentationAdmissionKey {
            attempt: basis.attempt(),
            binding: basis.binding(),
        };
        self.validate_admission_key(key)?;
        let lineage = super::super::semantic_transition::PresentationLineageKey::from_basis(&basis);
        self.validate_lineage_admission(lineage)?;
        let plan = self.plan_pending_admission(key, lineage, basis)?;
        let (key, pending) = self.admit_runtime_pending(plan)?;
        self.observe_and_finish_pending(key, pending)
    }

    fn validate_admission_key(
        &self,
        key: PresentationAdmissionKey,
    ) -> Result<(), WorthUiPresentationPendingAdmissionDenial> {
        if self.active_keys.contains(&key) {
            return Err(WorthUiPresentationPendingAdmissionDenial::DuplicateAttemptBinding);
        }
        if self.pending.len()
            + self.settling.len()
            + self.superseded_pending.len()
            + self.superseded_awaiting_completion.len()
            + self.runtime_cleanups.len()
            + self.unresolved.len()
            + self.terminal_closing.len()
            >= WORTH_UI_PRESENTATION_PENDING_CAPACITY
        {
            return Err(WorthUiPresentationPendingAdmissionDenial::PendingCapacityExhausted);
        }
        Ok(())
    }

    fn validate_lineage_admission(
        &self,
        lineage: super::super::semantic_transition::PresentationLineageKey,
    ) -> Result<(), WorthUiPresentationPendingAdmissionDenial> {
        if self
            .unresolved
            .values()
            .any(|pending| pending.lineage == lineage && !pending.recovery_required)
        {
            return Err(WorthUiPresentationPendingAdmissionDenial::UnresolvedLineageAdmission);
        }
        if self
            .settling
            .values()
            .any(|pending| pending.lineage == lineage)
        {
            return Err(WorthUiPresentationPendingAdmissionDenial::SettlingLineageAdmission);
        }
        if self.pending.values().any(|pending| {
            pending.lineage == lineage && !pending_progress::pending_admission_complete(pending)
        }) {
            return Err(WorthUiPresentationPendingAdmissionDenial::IncompleteLineageAdmission);
        }
        Ok(())
    }

    fn plan_pending_admission(
        &self,
        key: PresentationAdmissionKey,
        lineage: super::super::semantic_transition::PresentationLineageKey,
        basis: super::super::WorthUiPresentationRequestBasis,
    ) -> Result<PendingAdmissionPlan, WorthUiPresentationPendingAdmissionDenial> {
        let superseding_pending_predecessor = self
            .pending
            .values()
            .any(|pending| pending.lineage == lineage);
        let pending_predecessor = self.pending_predecessor(lineage);
        let reconstructing_unresolved_predecessor =
            pending_predecessor.is_some_and(|(_, recovery_required)| recovery_required);
        let transition_source = pending_predecessor
            .map(|(predecessor, _)| predecessor)
            .or_else(|| self.retained.get(&lineage));
        let transition = if reconstructing_unresolved_predecessor {
            super::super::semantic_transition::PresentationSemanticTransition::plan_reconstruction(
                transition_source
                    .ok_or(WorthUiPresentationPendingAdmissionDenial::MissingSemanticBaseline)?,
                &basis,
            )
        } else {
            super::super::semantic_transition::PresentationSemanticTransition::plan(
                transition_source,
                &basis,
            )
        }
        .map_err(map_transition_denial)?;
        Ok(PendingAdmissionPlan {
            key,
            basis,
            retention: PendingAdmissionRetentionPlan {
                lineage,
                transition,
                superseding_pending_predecessor,
                reconstructing_unresolved_predecessor,
            },
        })
    }

    fn pending_predecessor(
        &self,
        lineage: super::super::semantic_transition::PresentationLineageKey,
    ) -> Option<(
        &super::super::semantic_transition::RetainedPresentationSemanticState,
        bool,
    )> {
        self.pending
            .values()
            .filter(|pending| pending.lineage == lineage)
            .max_by_key(|pending| pending.nonce)
            .map(|pending| (pending.transition.successor(), false))
            .or_else(|| {
                self.unresolved
                    .values()
                    .filter(|pending| pending.lineage == lineage && pending.recovery_required)
                    .max_by_key(|pending| pending.nonce)
                    .map(|pending| (pending.transition.successor(), true))
            })
            .or_else(|| {
                self.superseded_pending
                    .values()
                    .filter(|pending| pending.lineage == lineage)
                    .max_by_key(|pending| pending.nonce)
                    .map(|pending| (pending.transition.successor(), pending.recovery_required))
            })
    }

    fn admit_runtime_pending(
        &mut self,
        plan: PendingAdmissionPlan,
    ) -> Result<
        (PresentationAdmissionKey, PendingPresentationAdmission),
        WorthUiPresentationPendingAdmissionDenial,
    > {
        let PendingAdmissionPlan {
            key,
            basis,
            retention,
        } = plan;
        let receipt_nonce = self
            .next_receipt_nonce
            .checked_add(1)
            .ok_or(WorthUiPresentationPendingAdmissionDenial::TruthRevisionExhausted)?;
        let declaration = super::super::WorthUiPresentationAsyncDeclaration::declare(&basis)
            .map_err(|_| {
                WorthUiPresentationPendingAdmissionDenial::Runtime(
                    WorthUiPresentationAdmissionStop::RuntimeAdmission,
                )
            })?;
        let admission = match WorthUiPresentationRuntimeAdmission::admit_in_workspace(
            &mut self.workspace,
            declaration,
            &self.owned_async_source,
            self.product.as_ref(),
        ) {
            Ok(admission) => admission,
            Err(denial) => match denial.into_cleanup_required() {
                Ok((cleanup, _cause, cleanup_denial)) => {
                    self.next_receipt_nonce = receipt_nonce;
                    return Err(self.retain_runtime_admission_cleanup(
                        key,
                        receipt_nonce,
                        cleanup,
                        cleanup_denial,
                    ));
                }
                Err(_) => {
                    return Err(WorthUiPresentationPendingAdmissionDenial::Runtime(
                        WorthUiPresentationAdmissionStop::RuntimeAdmission,
                    ));
                }
            },
        };
        self.next_receipt_nonce = receipt_nonce;
        Ok(pending_from_runtime_admission(
            key,
            retention,
            receipt_nonce,
            admission,
        ))
    }

    fn retain_runtime_admission_cleanup(
        &mut self,
        key: PresentationAdmissionKey,
        receipt_nonce: u64,
        cleanup: super::super::runtime_bridge::WorthUiPresentationRuntimeCleanup,
        denial: super::super::runtime_bridge::WorthUiPresentationRuntimeCleanupDenial,
    ) -> WorthUiPresentationPendingAdmissionDenial {
        let recovery = WorthUiPresentationCleanupRecovery {
            authority: std::sync::Arc::clone(&self.correspondence_authority),
            attempt: key.attempt,
            binding: key.binding,
            nonce: receipt_nonce,
        };
        self.runtime_cleanups.insert(
            key,
            PendingRuntimeCleanup {
                nonce: receipt_nonce,
                cleanup,
            },
        );
        self.active_keys.insert(key);
        WorthUiPresentationPendingAdmissionDenial::CleanupProgress(
            Box::new(recovery),
            WorthUiPresentationCleanupProgress {
                cause: WorthUiPresentationAdmissionStop::RuntimeAdmission,
                stopped_at: runtime_cleanup_stop(denial),
            },
        )
    }

    fn observe_and_finish_pending(
        &mut self,
        key: PresentationAdmissionKey,
        pending: PendingPresentationAdmission,
    ) -> Result<WorthUiPresentationPendingReceipt, WorthUiPresentationPendingAdmissionDenial> {
        let observation = match pending.admission.observation(&self.workspace) {
            Ok(observation) => observation,
            Err(_) => {
                return Err(self.retire_unpublished_admission(
                    key,
                    pending,
                    WorthUiPresentationAdmissionStop::RuntimeObservation,
                ));
            }
        };
        if observation.posture() != WorthUiPresentationAsyncPosture::Pending {
            return Err(self.retire_unpublished_admission(
                key,
                pending,
                WorthUiPresentationAdmissionStop::UnexpectedPendingPosture,
            ));
        }
        self.finish_pending_admission(key, pending, observation)
    }

    fn retire_unpublished_admission(
        &mut self,
        key: PresentationAdmissionKey,
        mut pending: PendingPresentationAdmission,
        cause: WorthUiPresentationAdmissionStop,
    ) -> WorthUiPresentationPendingAdmissionDenial {
        match self.advance_rejection(&mut pending) {
            Ok(()) => WorthUiPresentationPendingAdmissionDenial::Runtime(cause),
            Err(cleanup) => {
                let recovery = WorthUiPresentationCleanupRecovery {
                    authority: std::sync::Arc::clone(&self.correspondence_authority),
                    attempt: key.attempt,
                    binding: key.binding,
                    nonce: pending.nonce,
                };
                self.pending.insert(key, pending);
                self.active_keys.insert(key);
                WorthUiPresentationPendingAdmissionDenial::CleanupProgress(
                    Box::new(recovery),
                    WorthUiPresentationCleanupProgress {
                        cause,
                        stopped_at: cleanup_stop(&cleanup),
                    },
                )
            }
        }
    }
}

struct PendingAdmissionPlan {
    key: PresentationAdmissionKey,
    basis: super::super::WorthUiPresentationRequestBasis,
    retention: PendingAdmissionRetentionPlan,
}

struct PendingAdmissionRetentionPlan {
    lineage: super::super::semantic_transition::PresentationLineageKey,
    transition: super::super::semantic_transition::PresentationSemanticTransition,
    superseding_pending_predecessor: bool,
    reconstructing_unresolved_predecessor: bool,
}

fn pending_from_runtime_admission(
    key: PresentationAdmissionKey,
    plan: PendingAdmissionRetentionPlan,
    receipt_nonce: u64,
    admission: WorthUiPresentationRuntimeAdmission,
) -> (PresentationAdmissionKey, PendingPresentationAdmission) {
    (
        key,
        PendingPresentationAdmission {
            nonce: receipt_nonce,
            lineage: plan.lineage,
            transition: plan.transition,
            admission,
            supersession_query_admitted: false,
            supersession_posture_observed: false,
            predecessor_supersession_complete: false,
            settlement: PresentationSettlementProgress::default(),
            rejection: PresentationRejectionProgress::default(),
            recovery_required: false,
            superseding_pending_predecessor: plan.superseding_pending_predecessor,
            reconstructing_unresolved_predecessor: plan.reconstructing_unresolved_predecessor,
        },
    )
}

fn runtime_cleanup_stop(
    denial: super::super::runtime_bridge::WorthUiPresentationRuntimeCleanupDenial,
) -> WorthUiPresentationRuntimeCleanupStop {
    match denial {
        super::super::runtime_bridge::WorthUiPresentationRuntimeCleanupDenial::Query(_) => {
            WorthUiPresentationRuntimeCleanupStop::Query
        }
    }
}

fn cleanup_stop(
    denial: &WorthUiPresentationSettlementDenial,
) -> WorthUiPresentationRuntimeCleanupStop {
    match denial {
        WorthUiPresentationSettlementDenial::RuntimeCleanup(stop) => *stop,
        _ => WorthUiPresentationRuntimeCleanupStop::Query,
    }
}

fn map_transition_denial(
    denial: super::super::semantic_transition::PresentationSemanticTransitionDenial,
) -> WorthUiPresentationPendingAdmissionDenial {
    use super::super::semantic_transition::PresentationSemanticTransitionDenial as Denial;
    match denial {
        Denial::MissingBaseline => {
            WorthUiPresentationPendingAdmissionDenial::MissingSemanticBaseline
        }
        Denial::StalePredecessor => WorthUiPresentationPendingAdmissionDenial::StalePredecessor,
        Denial::ForeignHostSurface => WorthUiPresentationPendingAdmissionDenial::ForeignHostSurface,
        Denial::UnknownRemovedMechanic => {
            WorthUiPresentationPendingAdmissionDenial::UnknownRemovedMechanic
        }
        Denial::UnknownReleasedPin => WorthUiPresentationPendingAdmissionDenial::UnknownReleasedPin,
    }
}
