impl super::UiPortalRuntimeState {
    pub(crate) fn commit_published(
        &mut self,
        transition: super::super::UiPreparedPortalServiceTransition,
    ) -> Result<super::super::UiPortalServiceReceipt, super::super::UiPortalServiceTransitionDenial>
    {
        self.commit_published_with_exit_retention(transition, false)
            .map(|(receipt, _)| receipt)
    }

    pub(crate) fn commit_published_with_exit_retention(
        &mut self,
        transition: super::super::UiPreparedPortalServiceTransition,
        retain_exit: bool,
    ) -> Result<
        (
            super::super::UiPortalServiceReceipt,
            Option<super::super::UiPortalExitRetentionReceipt>,
        ),
        super::super::UiPortalServiceTransitionDenial,
    > {
        self.require_current(&transition)?;
        if transition.opens_portal() && !transition.is_idempotent() {
            let ordinal = transition
                .stack_ordinal()
                .expect("a fresh open retains its prepared stack ordinal");
            if !self.stack_order.can_insert(ordinal, transition.portal()) {
                return Err(super::super::UiPortalServiceTransitionDenial::StackOrdinalConflict);
            }
            self.stack_ordinal_issuer.advance(ordinal).map_err(|_| {
                super::super::UiPortalServiceTransitionDenial::StackOrdinalExhausted
            })?;
        }
        let existing_retention = self
            .records
            .get(&transition.portal())
            .and_then(|record| record.exit_retention);
        if transition.is_idempotent() {
            return self
                .commit(transition, None, existing_retention)
                .map(|receipt| (receipt, existing_retention));
        }
        let posture = match transition.request().operation() {
            super::super::request::UiPortalServiceOperation::Open => {
                super::super::UiPortalLifecyclePosture::Visible
            }
            super::super::request::UiPortalServiceOperation::Close(_) if retain_exit => {
                super::super::UiPortalLifecyclePosture::Closing
            }
            super::super::request::UiPortalServiceOperation::Close(_) => {
                super::super::UiPortalLifecyclePosture::Closed
            }
        };
        let exit_retention =
            (posture == super::super::UiPortalLifecyclePosture::Closing).then(|| {
                super::super::UiPortalExitRetentionReceipt::new(
                    transition.portal(),
                    transition.committed_revision(),
                    transition.request().idempotency().lineage(),
                )
            });
        self.commit(transition, Some(posture), exit_retention)
            .map(|receipt| (receipt, exit_retention))
    }

    pub(crate) fn prepare_exit_terminal(
        &self,
        retention: super::super::UiPortalExitRetentionReceipt,
        idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    ) -> Result<
        super::super::UiPreparedPortalServiceTransition,
        super::super::UiPortalExitTerminalDenial,
    > {
        let record = self
            .records
            .get(&retention.portal())
            .filter(|record| {
                record.posture == super::super::UiPortalLifecyclePosture::Closing
                    && record.exit_retention == Some(retention)
            })
            .ok_or(super::super::UiPortalExitTerminalDenial::RetentionMismatch)?;
        self.prepare(super::super::UiPortalServiceRequest::close(
            retention.portal(),
            idempotency,
            record
                .dismissal
                .expect("a Closing portal retains its dismissal cause"),
            record.semantic_surface,
        ))
        .map_err(super::super::UiPortalExitTerminalDenial::Transition)
    }

    pub(crate) fn validate_prepared(
        &self,
        transition: &super::super::UiPreparedPortalServiceTransition,
    ) -> Result<(), super::super::UiPortalServiceTransitionDenial> {
        self.require_current(transition)
    }

    fn commit(
        &mut self,
        transition: super::super::UiPreparedPortalServiceTransition,
        posture: Option<super::super::UiPortalLifecyclePosture>,
        exit_retention: Option<super::super::UiPortalExitRetentionReceipt>,
    ) -> Result<super::super::UiPortalServiceReceipt, super::super::UiPortalServiceTransitionDenial>
    {
        self.require_current(&transition)?;
        let prior_target_ordinal = self
            .records
            .get(&transition.portal())
            .map(|record| record.stack_ordinal);
        let request = transition.request();
        let policy = if transition.is_idempotent() {
            self.records
                .get(&transition.portal())
                .map_or(transition.policy(), |record| record.policy)
        } else {
            transition.policy()
        };
        let posture = posture.unwrap_or(transition.staged_posture());
        assert!(
            posture != super::super::UiPortalLifecyclePosture::Closing || exit_retention.is_some(),
            "a Closing Portal row must retain its terminal receipt"
        );
        let dismissal = match request.operation() {
            super::super::request::UiPortalServiceOperation::Open => None,
            super::super::request::UiPortalServiceOperation::Close(cause) => Some(cause),
        };
        if matches!(
            posture,
            super::super::UiPortalLifecyclePosture::Closed
                | super::super::UiPortalLifecyclePosture::Closing
        ) {
            for descendant in transition.closed_descendants().iter().copied() {
                let mut record = self
                    .records
                    .remove(&descendant)
                    .expect("prepared descendant closure retains its portal record");
                assert!(
                    record.exit_retention.is_none(),
                    "parent close cannot clear a descendant exit receipt"
                );
                record.posture = super::super::UiPortalLifecyclePosture::Closed;
                record.dismissal = Some(super::super::UiPortalDismissalCause::ParentClosed);
                record.placement = None;
                self.stack_order.remove(record.stack_ordinal, descendant);
                self.retain_record(descendant, record);
            }
        }
        let placement = transition
            .placement()
            .map(super::super::UiCommittedPortalPlacement::from_prepared)
            .or_else(|| {
                (posture == super::super::UiPortalLifecyclePosture::Closing)
                    .then(|| self.records.get(&request.portal())?.placement)
                    .flatten()
            });
        let stack_ordinal = transition.stack_ordinal().or_else(|| {
            self.records
                .get(&request.portal())
                .map(|record| record.stack_ordinal)
        });
        if let Some(stack_ordinal) = stack_ordinal {
            if posture == super::super::UiPortalLifecyclePosture::Closed {
                if let Some(previous) = prior_target_ordinal {
                    self.stack_order.remove(previous, request.portal());
                }
            } else if prior_target_ordinal != Some(stack_ordinal) {
                if let Some(previous) = prior_target_ordinal {
                    self.stack_order.remove(previous, request.portal());
                }
                self.stack_order.insert(stack_ordinal, request.portal());
            }
            self.retain_committed_record(
                request,
                super::UiPortalRecord {
                    policy,
                    posture,
                    semantic_surface: request.semantic_surface(),
                    last_request: request.idempotency(),
                    dismissal,
                    placement,
                    stack_ordinal,
                    exit_retention,
                },
            );
        } else {
            debug_assert_eq!(posture, super::super::UiPortalLifecyclePosture::Closed);
            self.closed_requests.retain(
                request.portal(),
                request.semantic_surface(),
                request.idempotency(),
                dismissal.expect("a terminal close retains its dismissal cause"),
            );
        }
        self.admitted_requests = self.admitted_requests.saturating_add(1);
        if transition.disposition() == super::super::UiPortalServiceDisposition::Idempotent {
            self.idempotent_requests = self.idempotent_requests.saturating_add(1);
        }
        self.revision = transition.committed_revision();
        if let super::super::request::UiPortalServiceOperation::Close(cause) = request.operation() {
            self.last_closed = Some(super::super::UiPortalClosedInspectionRecord::new(
                request.portal(),
                cause,
                transition.closed_descendants().len(),
                self.revision,
            ));
        }
        Ok(super::super::UiPortalServiceReceipt::new(
            request.portal(),
            posture,
            transition.disposition(),
        ))
    }

    fn retain_record(
        &mut self,
        portal: super::super::UiPortalIdentity,
        record: super::UiPortalRecord,
    ) {
        self.refresh_surface_stack(portal, &record);
        if record.posture == super::super::UiPortalLifecyclePosture::Closed {
            self.records.remove(&portal);
            self.closed_requests.retain(
                portal,
                record.semantic_surface,
                record.last_request,
                record
                    .dismissal
                    .expect("a Closed portal record retains its dismissal cause"),
            );
        } else {
            self.closed_requests.forget(portal);
            self.records.insert(portal, record);
        }
    }

    fn retain_committed_record(
        &mut self,
        request: super::super::UiPortalServiceRequest,
        record: super::UiPortalRecord,
    ) {
        self.retain_record(request.portal(), record);
    }

    fn require_current(
        &self,
        transition: &super::super::UiPreparedPortalServiceTransition,
    ) -> Result<(), super::super::UiPortalServiceTransitionDenial> {
        if transition.expected_revision() == self.revision {
            Ok(())
        } else {
            Err(super::super::UiPortalServiceTransitionDenial::StalePlan)
        }
    }
}
