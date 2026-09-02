impl super::UiPortalRuntimeState {
    pub(crate) fn prepare(
        &self,
        request: super::super::UiPortalServiceRequest,
    ) -> Result<
        super::super::UiPreparedPortalServiceTransition,
        super::super::UiPortalServiceTransitionDenial,
    > {
        let request = request.with_policy(self.policy);
        let committed_revision = self
            .revision
            .checked_add(1)
            .ok_or(super::super::UiPortalServiceTransitionDenial::RevisionExhausted)?;
        let existing = self.records.get(&request.portal());
        if let Some(record) = existing {
            if record.semantic_surface != request.semantic_surface() {
                return Err(super::super::UiPortalServiceTransitionDenial::PortalSurfaceMismatch);
            }
        }
        let parent = request
            .parent()
            .filter(|parent| *parent != request.portal())
            .and_then(|parent| self.records.get(&parent))
            .and_then(|record| record.placement);
        let placement = super::super::UiPreparedPortalPlacement::for_request(&request, parent)
            .map_err(super::super::UiPortalServiceTransitionDenial::Placement)?;
        let (staged_posture, disposition) = super::duplicate_request::classify(
            self.prior_request(request.portal()),
            &request,
            placement,
        );
        self.validate_operation(&request, disposition, placement)?;
        let staged_stack_ordinal = match (request.operation(), disposition) {
            (
                super::super::request::UiPortalServiceOperation::Open,
                super::super::UiPortalServiceDisposition::Idempotent,
            ) => self
                .records
                .get(&request.portal())
                .map(|record| record.stack_ordinal),
            (super::super::request::UiPortalServiceOperation::Open, _) => {
                let ordinal = self
                    .stack_ordinal_issuer
                    .next()
                    .ok_or(super::super::UiPortalServiceTransitionDenial::StackOrdinalExhausted)?;
                self.stack_order
                    .can_insert(ordinal, request.portal())
                    .then_some(ordinal)
                    .ok_or(super::super::UiPortalServiceTransitionDenial::StackOrdinalConflict)
                    .map(Some)?
            }
            (super::super::request::UiPortalServiceOperation::Close(_), _) => None,
        };
        let closed_descendants: Vec<_> = match (request.operation(), disposition) {
            (super::super::request::UiPortalServiceOperation::Open, _) => Vec::new(),
            (
                super::super::request::UiPortalServiceOperation::Close(_),
                super::super::UiPortalServiceDisposition::Idempotent,
            ) => Vec::new(),
            (super::super::request::UiPortalServiceOperation::Close(_), _) => self
                .records
                .keys()
                .copied()
                .filter(|portal| self.portal_descends_from(*portal, request.portal()))
                .collect::<Vec<_>>(),
        };
        if closed_descendants.iter().any(|portal| {
            self.records
                .get(portal)
                .is_some_and(|record| record.exit_retention.is_some())
        }) {
            return Err(
                super::super::UiPortalServiceTransitionDenial::DescendantExitRetentionPending,
            );
        }
        Ok(super::super::UiPreparedPortalServiceTransition::new(
            request,
            self.revision,
            committed_revision,
            staged_posture,
            disposition,
            placement,
            staged_stack_ordinal,
            closed_descendants.into_boxed_slice(),
        ))
    }

    fn validate_operation(
        &self,
        request: &super::super::UiPortalServiceRequest,
        disposition: super::super::UiPortalServiceDisposition,
        placement: Option<super::super::UiPreparedPortalPlacement>,
    ) -> Result<(), super::super::UiPortalServiceTransitionDenial> {
        let existing = self.records.get(&request.portal());
        match request.operation() {
            super::super::request::UiPortalServiceOperation::Open => {
                if disposition == super::super::UiPortalServiceDisposition::Idempotent {
                    return Ok(());
                }
                if let Some(record) = existing {
                    let same_parent = record
                        .placement
                        .and_then(|current| current.prepared().layer().parent())
                        == placement.and_then(|next| next.layer().parent());
                    if self.stack_order.topmost() != Some(request.portal())
                        || record.posture == super::super::UiPortalLifecyclePosture::Closing
                        || !same_parent
                    {
                        return Err(
                            super::super::UiPortalServiceTransitionDenial::ReplacementNotTopmost,
                        );
                    }
                } else if !super::super::capacity::admits_new_live_row(self.records.len()) {
                    return Err(
                        super::super::UiPortalServiceTransitionDenial::LiveRowCapacityExceeded {
                            limit: super::super::capacity::live_row_limit(),
                        },
                    );
                }
            }
            super::super::request::UiPortalServiceOperation::Close(_) => {
                if existing.is_some() {
                    return Ok(());
                }
                let Some(prior) = self.closed_requests.prior_request(request.portal()) else {
                    return Err(super::super::UiPortalServiceTransitionDenial::PortalNotLive);
                };
                if prior.semantic_surface != request.semantic_surface() {
                    return Err(
                        super::super::UiPortalServiceTransitionDenial::PortalSurfaceMismatch,
                    );
                }
                if disposition != super::super::UiPortalServiceDisposition::Idempotent {
                    return Err(super::super::UiPortalServiceTransitionDenial::PortalNotLive);
                }
            }
        }
        Ok(())
    }

    fn prior_request(
        &self,
        portal: super::super::UiPortalIdentity,
    ) -> Option<super::duplicate_request::UiPortalPriorRequest> {
        self.records
            .get(&portal)
            .map(|record| super::duplicate_request::UiPortalPriorRequest {
                posture: record.posture,
                semantic_surface: record.semantic_surface,
                last_request: record.last_request,
                dismissal: record.dismissal,
                placement: record.placement,
            })
            .or_else(|| self.closed_requests.prior_request(portal))
    }
}
