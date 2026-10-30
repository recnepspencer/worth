#[allow(unused_imports)]
use crate::surface_family_isolation::ordinary_context::*;

#[test]
fn surface_families_are_independently_inspectable_through_surface_roots() {
    let worth_native_only =
        test_server_with_surfaces(&[worth_server::WorthServerSurfaceFamily::WorthNative]);
    let surfaces = worth_native_only.surfaces();

    assert!(surfaces.worth_native().capabilities().is_registered());
    assert!(surfaces.worth_native().capabilities().is_disabled());
    assert!(!surfaces.worth_native().capabilities().is_absent());

    assert!(!surfaces.compat_http().capabilities().is_registered());
    assert!(surfaces.compat_http().capabilities().is_absent());
    assert!(!surfaces.compat_http().capabilities().is_disabled());

    assert!(surfaces.sync().capabilities().is_absent());
    assert!(surfaces.lease().capabilities().is_absent());
    assert!(surfaces.binary().capabilities().is_absent());
    assert!(surfaces.integration().capabilities().is_absent());
}

#[test]
fn sibling_surface_membership_does_not_change_shared_pipeline_truth() {
    let worth_native_only =
        test_server_with_surfaces(&[worth_server::WorthServerSurfaceFamily::WorthNative]);
    let with_compat_http = test_server_with_surfaces(&[
        worth_server::WorthServerSurfaceFamily::WorthNative,
        worth_server::WorthServerSurfaceFamily::CompatHttp,
    ]);

    let worth_native_only_context = resolve_worth_native_request_context(&worth_native_only);
    let with_compat_http_context = resolve_worth_native_request_context(&with_compat_http);

    assert_eq!(worth_native_only_context, with_compat_http_context);

    let worth_native_only_admission =
        worth_native_only
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                worth_native_only_context,
                worth_server::WorthServerPipelineIntent::query_read("users.profile"),
            ));
    let with_compat_http_admission =
        with_compat_http
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                with_compat_http_context,
                worth_server::WorthServerPipelineIntent::query_read("users.profile"),
            ));

    assert_eq!(worth_native_only_admission, with_compat_http_admission);
    assert!(matches!(
        worth_native_only_admission,
        worth_proof::TransitionOutcome::Success(_)
    ));
}

#[test]
fn future_surface_placeholder_registration_is_independent_of_present_family_truth() {
    let with_sync_placeholder = test_server_with_surfaces(&[
        worth_server::WorthServerSurfaceFamily::WorthNative,
        worth_server::WorthServerSurfaceFamily::Sync,
    ]);

    let surfaces = with_sync_placeholder.surfaces();
    assert!(surfaces.worth_native().capabilities().is_registered());
    assert!(surfaces.sync().capabilities().is_registered());
    assert!(surfaces.sync().capabilities().is_disabled());
    assert!(!surfaces.sync().capabilities().is_absent());
    assert!(surfaces.compat_http().capabilities().is_absent());

    let resolved_request_context = resolve_worth_native_request_context(&with_sync_placeholder);
    let admission =
        with_sync_placeholder
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                resolved_request_context,
                worth_server::WorthServerPipelineIntent::query_read("users.profile"),
            ));

    assert!(matches!(
        admission,
        worth_proof::TransitionOutcome::Success(_)
    ));
}
