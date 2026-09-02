use worth_ui_dsl::{UiBackdropIdentity, UiSemanticSurfaceDeclarationIdentity};

use super::backdrop_digest_support::{
    backdrop_role, backdrop_theme_view, current_value, declaration, inputs, overlay, projection_for,
};
use super::{
    UiAppearanceResolutionDenial, UiAppearanceResolutionSubject, UiAppearanceResolver,
    UiBackdropInstanceIdentity,
};

#[test]
fn identical_backdrop_inputs_are_exactly_equivalent_and_digest_identical() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let first = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        let second = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        assert!(first.exactly_equivalent(&second));
        assert_eq!(first.semantic_digest(), second.semantic_digest());
        assert_eq!(first.catalog_revision(), theme.catalog_revision());
        assert_eq!(first.aspects().len(), 2);
        let _ = session.shutdown();
    });
}

#[test]
fn backdrop_digest_carries_instance_declaration_and_overlay_evidence() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let base = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        let changed_instance = projection_for(&session, &role, surface, &vector, &theme, 8, 1, 1);
        let changed_overlay = projection_for(&session, &role, surface, &vector, &theme, 7, 2, 1);
        let changed_declaration_revision =
            projection_for(&session, &role, surface, &vector, &theme, 7, 1, 2);
        for changed in [
            changed_instance,
            changed_overlay,
            changed_declaration_revision,
        ] {
            assert_ne!(base.semantic_digest(), changed.semantic_digest());
            assert!(!base.exactly_equivalent(&changed));
        }
        let _ = session.shutdown();
    });
}

#[test]
fn backdrop_denial_evidence_is_resolver_owned_and_effect_free() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(9).unwrap();
        let declaration = declaration(&role, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(
            &session,
            surface,
            declaration_surface,
            &declaration,
            instance,
            false,
            1,
            1,
        );
        let before_value = current_value(&session);
        let before_frames = session.inspect_mounted_identity().frame_receipts().len();

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(instance, &declaration, &role, &vector, &theme, &overlay)
            .expect_err("a missing overlay participant must be denied by the resolver");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::OverlayParticipantMissing
        );
        assert_eq!(
            evidence.subject(),
            UiAppearanceResolutionSubject::Backdrop(instance)
        );
        assert_ne!(evidence.input_digest(), 0);
        assert_eq!(current_value(&session), before_value);
        assert_eq!(
            session.inspect_mounted_identity().frame_receipts().len(),
            before_frames
        );
        let _ = session.shutdown();
    });
}

#[test]
fn backdrop_surface_denial_has_independent_zero_effect_evidence() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, _theme) = inputs();
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(10).unwrap();
        let declaration = declaration(&role, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(
            &session,
            surface,
            declaration_surface,
            &declaration,
            instance,
            true,
            1,
            1,
        );
        let foreign_surface =
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let foreign_theme = backdrop_theme_view(&session, &role, foreign_surface);
        let before_value = current_value(&session);

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(
                instance,
                &declaration,
                &role,
                &vector,
                &foreign_theme,
                &overlay,
            )
            .expect_err("a vector/theme surface mismatch must be denied");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::WrongSurface
        );
        assert_eq!(current_value(&session), before_value);
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}

#[test]
fn unadmitted_backdrop_role_denial_is_independent_and_effect_free() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, _role, surface, vector, theme) = inputs();
        let unadmitted = backdrop_role("test.backdrop.unadmitted");
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(11).unwrap();
        let declaration = declaration(&unadmitted, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(
            &session,
            surface,
            declaration_surface,
            &declaration,
            instance,
            true,
            1,
            1,
        );
        let before_value = current_value(&session);

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(
                instance,
                &declaration,
                &unadmitted,
                &vector,
                &theme,
                &overlay,
            )
            .expect_err("an unadmitted role must be denied by the resolver");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::MissingRoleCapability
        );
        assert_eq!(current_value(&session), before_value);
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}
