use super::compat_http_phase_five_runtime::{
    build_phase_five_server, compat_upload_execution_input, compat_upload_success,
    upload_order_alpha,
};

#[test]
fn compat_http_upload_keeps_blob_transport_out_of_structured_truth_artifacts() {
    let server = build_phase_five_server();
    let upload = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "files.avatar.upload",
        "boundary-blob",
        upload_order_alpha("task-11"),
    )));

    assert!(upload.upload().canonical_digest().contains("avatar"));
    assert!(upload.upload().canonical_digest().contains("image/png"));
    assert!(!upload
        .mutation()
        .mutation_request()
        .canonical_digest()
        .contains("avatar"));
    assert!(!upload
        .mutation()
        .envelope()
        .response_envelope()
        .canonical_digest()
        .contains("image/png"));
    assert!(!upload.mutation().canonical_digest().contains("thumbnail"));
}
