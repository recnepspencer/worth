use super::compat_http_phase_five_runtime;
use super::compat_http_phase_five_runtime::{
    build_phase_five_server, compat_mutation_execution_input, compat_mutation_success,
    compat_upload_execution_input, compat_upload_success, prepared_request, single_insert_body,
    upload_order_alpha, upload_order_beta,
};

#[test]
fn compat_http_upload_preserves_canonical_metadata_truth_across_part_order_and_boundary_variation()
{
    let alpha_server = build_phase_five_server();
    let beta_server = build_phase_five_server();

    let alpha_prepared = prepared_request(
        &alpha_server,
        compat_http_phase_five_runtime::upload_input("files.avatar.upload", "boundary-alpha")
            .build()
            .expect("alpha upload input should validate structurally"),
    );
    let beta_prepared = prepared_request(
        &beta_server,
        compat_http_phase_five_runtime::upload_input("files.avatar.upload", "boundary-beta")
            .build()
            .expect("beta upload input should validate structurally"),
    );
    let alpha = compat_upload_success(alpha_server.compat_http().upload(
        compat_upload_execution_input(
            &alpha_server,
            "files.avatar.upload",
            "boundary-alpha",
            upload_order_alpha("task-1"),
        ),
    ));
    let beta = compat_upload_success(beta_server.compat_http().upload(
        compat_upload_execution_input(
            &beta_server,
            "files.avatar.upload",
            "boundary-beta",
            upload_order_beta("task-1"),
        ),
    ));

    assert_ne!(
        alpha_prepared.request_contract().canonical_digest(),
        beta_prepared.request_contract().canonical_digest()
    );
    assert_eq!(
        alpha.upload().canonical_digest(),
        beta.upload().canonical_digest()
    );
    assert_eq!(
        alpha.mutation().mutation_request().canonical_digest(),
        beta.mutation().mutation_request().canonical_digest()
    );
    assert_eq!(
        alpha.mutation().mutation_result().result_digest(),
        beta.mutation().mutation_result().result_digest()
    );
    assert_eq!(
        alpha.mutation().mutation_result().inspection_digest(),
        beta.mutation().mutation_result().inspection_digest()
    );
    assert_eq!(
        alpha
            .mutation()
            .envelope()
            .response_envelope()
            .canonical_digest(),
        beta.mutation()
            .envelope()
            .response_envelope()
            .canonical_digest()
    );
}

#[test]
fn compat_http_upload_lowers_metadata_through_the_same_mutation_lane() {
    let upload_server = build_phase_five_server();
    let mutation_server = build_phase_five_server();

    let upload = compat_upload_success(upload_server.compat_http().upload(
        compat_upload_execution_input(
            &upload_server,
            "files.avatar.upload",
            "boundary-shared",
            upload_order_alpha("task-7"),
        ),
    ));
    let plain_mutation = compat_mutation_success(mutation_server.compat_http().mutate(
        compat_mutation_execution_input(
            &mutation_server,
            "files.avatar.upload",
            single_insert_body("task-7"),
        ),
    ));

    assert_eq!(
        upload.mutation().mutation_request().canonical_digest(),
        plain_mutation.mutation_request().canonical_digest()
    );
    assert_eq!(
        upload.mutation().mutation_result().result_digest(),
        plain_mutation.mutation_result().result_digest()
    );
    assert_eq!(
        upload.mutation().mutation_result().inspection_digest(),
        plain_mutation.mutation_result().inspection_digest()
    );
}
