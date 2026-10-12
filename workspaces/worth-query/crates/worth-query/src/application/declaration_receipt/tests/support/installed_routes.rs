use super::*;

pub(crate) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        GeometryWorld::named(regime),
        [
            crate::application::domain_test_support::family::<
                GeometryDomain,
                RelationalReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, MixedReceiptFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                RequiredIntentReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredReceiptFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ForbiddenIntentReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, FailedReceiptFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, SignalReceiptFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                AspectRichReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                AspectDeferredReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                AspectSignalReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                AspectFailedReceiptFamily,
            >(),
        ],
    )
}

pub(crate) fn progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: ReceiptInput<F>,
) -> crate::application::WorthQueryAdmittedDeclarationProgression<GeometryDomain, ReceiptInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    ReceiptInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("receipt progression should admit"))
}

pub(crate) fn foundational_from_progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    progression: crate::application::WorthQueryAdmittedDeclarationProgression<
        GeometryDomain,
        ReceiptInput<F>,
    >,
) -> crate::application::WorthQueryDeclarationFoundationalEvidence<GeometryDomain, ReceiptInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    ReceiptInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    handle
        .describe_foundational(
            crate::application::WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progression,
            ),
        )
        .unwrap_or_else(|_| panic!("foundational evidence should materialize"))
}

pub(crate) fn route_checked_from_input<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: ReceiptInput<F>,
) -> crate::application::WorthQueryDeclarationRoutePlanChecked<GeometryDomain, ReceiptInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    ReceiptInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    let progression = progressed(handle, declaration);
    let evidence = foundational_from_progressed(handle, progression.clone());
    handle.plan_routes_checked(WorthQueryDeclarationRoutePlanInput::admitted(
        progression,
        evidence,
    ))
}

pub(crate) fn route_checked_with_intent<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: ReceiptInput<F>,
    intent: WorthQueryDeclarationRouteIntent,
) -> crate::application::WorthQueryDeclarationRoutePlanChecked<GeometryDomain, ReceiptInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    ReceiptInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    let progression = progressed(handle, declaration);
    let evidence = foundational_from_progressed(handle, progression.clone());
    handle.plan_routes_checked(WorthQueryDeclarationRoutePlanInput::with_intent(
        progression,
        evidence,
        intent,
    ))
}
