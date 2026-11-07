use crate::application::{
    worth_query_checked_declaration_relational_routing_on_handle,
    WorthQueryDeclarationAuthorityAspectMismatch, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationFoundationalEvidenceInput, WorthQueryDeclarationInput,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRelationalRouting,
    WorthQueryDeclarationRelationalRoutingChecked, WorthQueryDeclarationRelationalRoutingInput,
    WorthQueryDeclarationRelationalRoutingSupportReport,
    WorthQueryDeclarationRelationalRoutingTerminalError,
    WorthQueryDeclarationRelationalTruthRoutingSupportStatus, WorthQueryDeclarationRouteIntent,
    WorthQueryDeclarationRoutePlanInput, WorthQueryDeclarationSupportsRelationalTruth,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn route_relational_truth<I: WorthQueryDeclarationInput<D>>(
        &self,
        subject: WorthQueryDeclarationRelationalRoutingInput<D, I>,
    ) -> Result<
        WorthQueryDeclarationRelationalRouting<D, I>,
        WorthQueryDeclarationRelationalRoutingTerminalError<D, I>,
    > {
        match self.route_relational_truth_checked(subject)? {
            WorthQueryDeclarationRelationalRoutingChecked::Routed(routing) => Ok(routing),
            WorthQueryDeclarationRelationalRoutingChecked::Deferred(routing) => {
                Err(WorthQueryDeclarationRelationalRoutingTerminalError::Deferred(routing))
            }
            WorthQueryDeclarationRelationalRoutingChecked::Denied(routing) => Err(
                WorthQueryDeclarationRelationalRoutingTerminalError::Denied(routing),
            ),
            WorthQueryDeclarationRelationalRoutingChecked::Failed(routing) => Err(
                WorthQueryDeclarationRelationalRoutingTerminalError::Failed(routing),
            ),
        }
    }

    pub fn route_relational_truth_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        subject: WorthQueryDeclarationRelationalRoutingInput<D, I>,
    ) -> Result<WorthQueryDeclarationRelationalRoutingChecked<D, I>, IdentityDerivationDenial> {
        let support_status = self
            .relational_truth_support::<I>()
            .rows()
            .first()
            .map(|row| {
                if row.status() == WorthQueryDeclarationRelationalTruthRoutingSupportStatus::Unsupported
                    && matches!(
                        row.aspect_mismatch(),
                        Some(
                            WorthQueryDeclarationAuthorityAspectMismatch::MissingRequiredAspect
                                | WorthQueryDeclarationAuthorityAspectMismatch::AspectConflict
                                | WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectGap
                                | WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectAmbiguity
                        )
                    )
                {
                    WorthQueryDeclarationRelationalTruthRoutingSupportStatus::Admitted
                } else {
                    row.status()
                }
            })
            .unwrap_or(WorthQueryDeclarationRelationalTruthRoutingSupportStatus::Unsupported);
        Ok(
            worth_query_checked_declaration_relational_routing_on_handle(
                self.handle_identity_digest(),
                self.operating_context_identity_digest(),
                support_status,
                subject,
            )?,
        )
    }

    pub fn relational_truth_support<I: WorthQueryDeclarationInput<D>>(
        &self,
    ) -> WorthQueryDeclarationRelationalRoutingSupportReport<D, I> {
        crate::application::derive_relational_routing_support_report::<D, C, I>(self)
    }
}

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn route_relational_truth_from_progressed<I>(
        &self,
        progressed: crate::application::WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryDeclarationRelationalRouting<D, I>,
        WorthQueryDeclarationRelationalRoutingTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsRelationalTruth<D>,
    {
        self.route_relational_truth_from_progressed_with_intent(progressed, None)
    }

    pub fn route_relational_truth_from_progressed_with_intent<I>(
        &self,
        progressed: crate::application::WorthQueryAdmittedDeclarationProgression<D, I>,
        intent: impl Into<Option<WorthQueryDeclarationRouteIntent>>,
    ) -> Result<
        WorthQueryDeclarationRelationalRouting<D, I>,
        WorthQueryDeclarationRelationalRoutingTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsRelationalTruth<D>,
    {
        let evidence = self
            .describe_foundational(
                WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                    progressed.clone(),
                ),
            )
            .unwrap_or_else(|_| panic!("same-handle foundational evidence should materialize"));
        let route_checked = match intent.into() {
            Some(intent) => self.plan_routes_checked(
                WorthQueryDeclarationRoutePlanInput::with_intent(progressed, evidence, intent),
            ),
            None => self.plan_routes_checked(WorthQueryDeclarationRoutePlanInput::admitted(
                progressed, evidence,
            )),
        };
        let receipt_checked = self.receipt_routes_checked(
            WorthQueryDeclarationReceiptInput::route_checked(route_checked),
        );
        let envelope_checked = self.envelope_routes_checked(
            WorthQueryDeclarationEnvelopeInput::receipt_checked(receipt_checked),
        );
        match self.route_relational_truth_checked(
            WorthQueryDeclarationRelationalRoutingInput::envelope_checked(envelope_checked),
        )? {
            WorthQueryDeclarationRelationalRoutingChecked::Routed(routing) => Ok(routing),
            WorthQueryDeclarationRelationalRoutingChecked::Deferred(routing) => {
                Err(WorthQueryDeclarationRelationalRoutingTerminalError::Deferred(routing))
            }
            WorthQueryDeclarationRelationalRoutingChecked::Denied(routing) => Err(
                WorthQueryDeclarationRelationalRoutingTerminalError::Denied(routing),
            ),
            WorthQueryDeclarationRelationalRoutingChecked::Failed(routing) => Err(
                WorthQueryDeclarationRelationalRoutingTerminalError::Failed(routing),
            ),
        }
    }

    pub fn declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryDeclarationRelationalRouting<D, I>,
        crate::application::WorthQueryDeclarationEntryRelationalRoutingError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsRelationalTruth<D>,
    {
        let progressed = self.declare_review_and_progress(input).map_err(|error| {
            crate::application::WorthQueryDeclarationEntryRelationalRoutingError::Entry(
                crate::application::WorthQueryDeclarationEntryEnvelopeError::Entry(
                    crate::application::WorthQueryDeclarationEntryReceiptError::Entry(error),
                ),
            )
        })?;
        self.route_relational_truth_from_progressed(progressed)
            .map_err(crate::application::WorthQueryDeclarationEntryRelationalRoutingError::Routing)
    }
}
