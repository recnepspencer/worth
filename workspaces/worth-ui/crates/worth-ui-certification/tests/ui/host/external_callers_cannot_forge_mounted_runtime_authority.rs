use worth_ui_host_contract::{UiHostPresentationCompletionToken, UiMountedFrameConsumptionView};
use worth_ui_runtime::facade::host::{
    UiHostAdapterSessionAuthority, WorthUiOperationalHostAdapter,
};
use worth_ui_runtime::facade::mounted::{
    UiMountedPresentationAttemptIdentity, UiMountedPresentationWitness,
};

fn mint_removed_runtime_authority() {
    let _ = worth_ui_host_contract::UiMountedPresentationAuthority::mint_for_runtime();
}

fn extract_runtime_authority(view: &UiMountedFrameConsumptionView<'_>) {
    let _ = view.application_authority();
    let _ = view.graph_authority();
    let _ = view.query_authority();
    let _ = view.publication_authority();
}

fn clone_completion_token(token: UiHostPresentationCompletionToken) {
    let _ = token.clone();
}

fn forge_publication_witness(attempt: UiMountedPresentationAttemptIdentity) {
    let _ = UiMountedPresentationWitness::new(attempt);
}

fn invoke_host_effect_without_runtime_session(
    adapter: &dyn WorthUiOperationalHostAdapter,
    request: worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
) {
    let forged = UiHostAdapterSessionAuthority::activate(request.host_session_identity());
    let _ = adapter.register_surface(&forged, request);
}

fn main() {
    let _ = (
        mint_removed_runtime_authority,
        extract_runtime_authority,
        clone_completion_token,
        forge_publication_witness,
        invoke_host_effect_without_runtime_session,
    );
}

mod incomplete_preparation {
    include!("incomplete_frame_cannot_reach_presentation.rs");
}
