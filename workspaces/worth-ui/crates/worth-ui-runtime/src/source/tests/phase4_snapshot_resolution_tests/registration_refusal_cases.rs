use crate::capability::{
    CapabilityDiagnosticCode, CapabilityRegistrationDiagnostic, CapabilityRegistrationRejections,
    CapabilitySupportCatalog, CapabilitySupportKind, RegistrationCandidate, COMPONENT_FAMILY_NAME,
    SURFACE_FAMILY_NAME, THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};
use crate::source::{WorthUiArtifactInputResolver, WorthUiResolutionDiagnosticCode};
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};

use super::resolution_fixture_support::{empty_snapshot, snapshot_with_registration_rejections};

const COMPONENT: &str = "workspace.component.refused";
const SURFACE: &str = "workspace.surface.refused";
const VIEW_BINDING: &str = "workspace.view_binding.refused";
const THEME_TOKEN: &str = "theme.text.refused";

#[test]
fn a_reference_to_a_refused_registration_is_rejected_with_its_refusal() {
    let snapshot = snapshot_with_registration_rejections(
        empty_snapshot().capabilities(),
        CapabilitySupportCatalog::default(),
        [
            (COMPONENT_FAMILY_NAME, COMPONENT),
            (SURFACE_FAMILY_NAME, SURFACE),
            (VIEW_BINDING_FAMILY_NAME, VIEW_BINDING),
            (THEME_TOKEN_FAMILY_NAME, THEME_TOKEN),
        ]
        .into_iter()
        .fold(
            CapabilityRegistrationRejections::default(),
            |rejections, (family, identity)| {
                rejections.with_refusal(family, identity, vec![refusal(family, identity)])
            },
        ),
    );

    let report = WorthUiArtifactInputResolver::resolve(&every_family_input(), &snapshot)
        .expect_err("references to refused registrations fail at phase 4");

    let resolved: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let [refusal] = diagnostic.registration() else {
                panic!("each rejection carries the one refusal");
            };
            (
                diagnostic.code(),
                diagnostic.authored_text(),
                refusal.family_name(),
                refusal.identity_text(),
            )
        })
        .collect();
    assert_eq!(
        resolved,
        [
            (
                WorthUiResolutionDiagnosticCode::RejectedComponentReference,
                COMPONENT,
                Some(COMPONENT_FAMILY_NAME),
                Some(COMPONENT),
            ),
            (
                WorthUiResolutionDiagnosticCode::RejectedSurfaceReference,
                SURFACE,
                Some(SURFACE_FAMILY_NAME),
                Some(SURFACE),
            ),
            (
                WorthUiResolutionDiagnosticCode::RejectedViewBindingReference,
                VIEW_BINDING,
                Some(VIEW_BINDING_FAMILY_NAME),
                Some(VIEW_BINDING),
            ),
            (
                WorthUiResolutionDiagnosticCode::RejectedThemeTokenReference,
                THEME_TOKEN,
                Some(THEME_TOKEN_FAMILY_NAME),
                Some(THEME_TOKEN),
            ),
        ]
    );
}

#[test]
fn a_refused_registration_with_a_declared_posture_keeps_that_posture() {
    let snapshot = snapshot_with_registration_rejections(
        empty_snapshot().capabilities(),
        CapabilitySupportCatalog::from_registration_candidates(&[
            RegistrationCandidate::with_support(
                COMPONENT_FAMILY_NAME,
                COMPONENT,
                CapabilitySupportKind::Deferred,
            ),
        ]),
        CapabilityRegistrationRejections::default().with_refusal(
            COMPONENT_FAMILY_NAME,
            COMPONENT,
            vec![refusal(COMPONENT_FAMILY_NAME, COMPONENT)],
        ),
    );
    let input = crate::source::test_compilation::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui").with_component(COMPONENT),
        ]),
    );

    let report = WorthUiArtifactInputResolver::resolve(&input, &snapshot)
        .expect_err("a deferred component fails at phase 4");

    let [diagnostic] = report.diagnostics() else {
        panic!("the one reference is refused");
    };
    assert_eq!(
        diagnostic.code(),
        WorthUiResolutionDiagnosticCode::DeferredComponentReference
    );
    assert_eq!(diagnostic.registration().len(), 1);
}

fn every_family_input() -> worth_ui_dsl::WorthUiSealedSemanticPackage {
    crate::source::test_compilation::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_component(COMPONENT)
                .with_surface(SURFACE)
                .with_binding(VIEW_BINDING)
                .with_token(THEME_TOKEN, THEME_TOKEN),
        ]),
    )
}

fn refusal(family: &'static str, identity: &str) -> CapabilityRegistrationDiagnostic {
    CapabilityRegistrationDiagnostic::error(
        CapabilityDiagnosticCode::DuplicateCapabilityId,
        Some(family),
        Some(identity),
        None,
        None,
        None,
    )
}
