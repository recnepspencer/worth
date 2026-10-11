//! A refused Query opening retires the already admitted presentation request.
use super::*;
use crate::presentation_async::{
    runtime_bridge::WorthUiPresentationRuntimeAdmissionDenial, WorthUiPresentationAsyncDeclaration,
};

#[test]
fn refused_live_opening_retires_the_admitted_request() {
    let mut installed = installed_owner();
    let declaration = WorthUiPresentationAsyncDeclaration::declare(&basis(0)).unwrap();
    let WorthUiPresentationAsyncOwner {
        workspace,
        product,
        owned_async_source,
        ..
    } = &mut installed.owner;
    // This request gives custody for inspecting all requests on the binding,
    // including its replacement that the failed admission cannot return.
    let probe = workspace
        .admit_owned_bridge_async_request(owned_async_source, product)
        .unwrap();
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(&probe)
            .unwrap(),
        1
    );
    workspace
        .with_advancement_mut(|_, workspace| {
            let Err(WorthUiPresentationRuntimeAdmissionDenial::QueryLive(denial)) =
                WorthUiPresentationRuntimeAdmission::admit_in_workspace(
                    workspace,
                    declaration,
                    owned_async_source,
                    product,
                )
            else {
                panic!("the nested opener must return its typed live denial")
            };
            let runtime::WorthQueryRuntimeError::Advancement(cause) = *denial else {
                panic!("the original opening refusal must survive")
            };
            assert_eq!(
                cause,
                worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::NestedOpening
            );
        })
        .unwrap();
    // Admission replaces the probe; refusing its opening must retire the new
    // request too, leaving no active request on this installed binding.
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(&probe)
            .unwrap(),
        0
    );
}
