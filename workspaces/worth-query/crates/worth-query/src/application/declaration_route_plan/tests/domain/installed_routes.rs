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
            crate::application::domain_test_support::family::<GeometryDomain, RelationalRouteFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, MixedRouteFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, RequiredIntentFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, ForbiddenIntentFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredRouteFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, FailedRouteFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, AspectRichRouteFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                MissingAspectRouteFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictAspectRouteFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                TemporalBridgeRouteFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncBridgeRouteFamily>(
            ),
        ],
    )
}

pub(crate) fn progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RouteInput<F>,
) -> crate::application::WorthQueryAdmittedDeclarationProgression<GeometryDomain, RouteInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RouteInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("route-plan progression should admit"))
}

pub(crate) fn route_input<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RouteInput<F>,
) -> WorthQueryDeclarationRoutePlanInput<GeometryDomain, RouteInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RouteInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    let progressed = progressed(handle, declaration);
    let evidence = handle
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progressed.clone(),
            ),
        )
        .unwrap_or_else(|_| panic!("same-handle foundational evidence should materialize"));
    WorthQueryDeclarationRoutePlanInput::admitted(progressed, evidence)
}

pub(crate) fn future_supported_route_input<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RouteInput<F>,
) -> WorthQueryDeclarationRoutePlanInput<GeometryDomain, RouteInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RouteInput<F>: WorthQueryDeclarationInput<GeometryDomain, Family = F>,
{
    let canonical = handle
        .declare(declaration.clone())
        .unwrap_or_else(|_| panic!("future declaration should canonicalize"));
    let support_report = handle.family_support::<F>();
    let legal = match crate::application::review_declaration_legality(
        handle.handle_identity_digest(),
        WorthQueryDeclarationLegalityInput::new(
            canonical,
            support_report,
            F::legality_contract(),
            handle.retained_world_basis(),
            Some(WorthQueryRuntimeFamilySupportStatus::Supported),
            Some(WorthQueryRuntimeFamilySupportStatus::Supported),
        ),
    ) {
        WorthQueryDeclarationLegalityChecked::Legal(legal) => legal,
        WorthQueryDeclarationLegalityChecked::Illegal(_) => {
            panic!("future declaration should become legal under supported runtime test posture")
        }
    };
    let progressed = handle
        .progress_declaration(legal)
        .unwrap_or_else(|_| panic!("future declaration progression should admit"));
    let evidence = handle
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progressed.clone(),
            ),
        )
        .unwrap_or_else(|_| panic!("future foundational evidence should materialize"));
    WorthQueryDeclarationRoutePlanInput::admitted(progressed, evidence)
}
