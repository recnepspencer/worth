use crate::application::{
    derive_bridge_routing_support_report, worth_query_checked_declaration_bridge_routing_on_handle,
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationAuthorityAspectMismatch,
    WorthQueryDeclarationBridgeRouting, WorthQueryDeclarationBridgeRoutingChecked,
    WorthQueryDeclarationBridgeRoutingInput, WorthQueryDeclarationBridgeRoutingSupportReport,
    WorthQueryDeclarationBridgeRoutingSupportStatus,
    WorthQueryDeclarationBridgeRoutingTerminalError, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationFoundationalEvidenceInput, WorthQueryDeclarationInput,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRouteIntent,
    WorthQueryDeclarationRoutePlanInput, WorthQueryDeclarationSupportsBridgeContinuation,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn route_bridge_continuation<I: WorthQueryDeclarationInput<D>>(
        &self,
        subject: WorthQueryDeclarationBridgeRoutingInput<D, I>,
    ) -> Result<
        WorthQueryDeclarationBridgeRouting<D, I>,
        WorthQueryDeclarationBridgeRoutingTerminalError<D, I>,
    > {
        match self.route_bridge_continuation_checked(subject) {
            WorthQueryDeclarationBridgeRoutingChecked::Routed(routing) => Ok(routing),
            WorthQueryDeclarationBridgeRoutingChecked::Deferred(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Deferred(routing),
            ),
            WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Denied(routing),
            ),
            WorthQueryDeclarationBridgeRoutingChecked::Failed(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Failed(routing),
            ),
        }
    }

    pub fn route_bridge_continuation_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        subject: WorthQueryDeclarationBridgeRoutingInput<D, I>,
    ) -> Result<WorthQueryDeclarationBridgeRoutingChecked<D, I>, IdentityDerivationDenial> {
        let support_status = self
            .bridge_continuation_support::<I>()
            .rows()
            .first()
            .map(|row| {
                if row.status() == WorthQueryDeclarationBridgeRoutingSupportStatus::Unsupported
                    && (matches!(
                        row.aspect_mismatch(),
                        Some(
                            WorthQueryDeclarationAuthorityAspectMismatch::MissingRequiredAspect
                                | WorthQueryDeclarationAuthorityAspectMismatch::AspectConflict
                                | WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectGap
                                | WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectAmbiguity
                        )
                    ) || !matches!(
                        row.mapping_fit(),
                        WorthQueryDeclarationAspectFit::Exact
                            | WorthQueryDeclarationAspectFit::CompatibleSuperset
                    ))
                {
                    WorthQueryDeclarationBridgeRoutingSupportStatus::Admitted
                } else {
                    row.status()
                }
            })
            .unwrap_or(WorthQueryDeclarationBridgeRoutingSupportStatus::Unsupported);
        Ok(worth_query_checked_declaration_bridge_routing_on_handle(
            self.handle_identity_digest(),
            self.operating_context_identity_digest(),
            support_status,
            subject,
        )?)
    }

    pub fn bridge_continuation_support<I: WorthQueryDeclarationInput<D>>(
        &self,
    ) -> WorthQueryDeclarationBridgeRoutingSupportReport<D, I> {
        derive_bridge_routing_support_report::<D, C, I>(self)
    }
}

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn route_bridge_continuation_from_progressed<I>(
        &self,
        progressed: crate::application::WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryDeclarationBridgeRouting<D, I>,
        WorthQueryDeclarationBridgeRoutingTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsBridgeContinuation<D>,
    {
        self.route_bridge_continuation_from_progressed_with_intent(progressed, None)
    }

    pub fn route_bridge_continuation_from_progressed_with_intent<I>(
        &self,
        progressed: crate::application::WorthQueryAdmittedDeclarationProgression<D, I>,
        intent: impl Into<Option<WorthQueryDeclarationRouteIntent>>,
    ) -> Result<
        WorthQueryDeclarationBridgeRouting<D, I>,
        WorthQueryDeclarationBridgeRoutingTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsBridgeContinuation<D>,
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
        match self.route_bridge_continuation_checked(
            WorthQueryDeclarationBridgeRoutingInput::envelope_checked(envelope_checked),
        ) {
            WorthQueryDeclarationBridgeRoutingChecked::Routed(routing) => Ok(routing),
            WorthQueryDeclarationBridgeRoutingChecked::Deferred(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Deferred(routing),
            ),
            WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Denied(routing),
            ),
            WorthQueryDeclarationBridgeRoutingChecked::Failed(routing) => Err(
                WorthQueryDeclarationBridgeRoutingTerminalError::Failed(routing),
            ),
        }
    }

    pub fn declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryDeclarationBridgeRouting<D, I>,
        crate::application::WorthQueryDeclarationEntryBridgeRoutingError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
        I::Family: WorthQueryDeclarationSupportsBridgeContinuation<D>,
    {
        let progressed = self.declare_review_and_progress(input).map_err(|error| {
            crate::application::WorthQueryDeclarationEntryBridgeRoutingError::Entry(
                crate::application::WorthQueryDeclarationEntryEnvelopeError::Entry(
                    crate::application::WorthQueryDeclarationEntryReceiptError::Entry(error),
                ),
            )
        })?;
        self.route_bridge_continuation_from_progressed(progressed)
            .map_err(crate::application::WorthQueryDeclarationEntryBridgeRoutingError::Routing)
    }
}
