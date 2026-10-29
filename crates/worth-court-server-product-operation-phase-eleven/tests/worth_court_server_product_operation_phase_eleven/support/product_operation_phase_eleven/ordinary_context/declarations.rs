pub(crate) use {worth_server::WorthServerCompatibilityAdmittedProductMutationCommand, worth_server::WorthServerProductIdempotencyKey, worth_server::WorthServerProductOperationBaseDigest, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerProductSessionCreationRequest, worth_server::WorthServerWorthNativeProductMutationCommand};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_operation_phase_eleven_backend as backend};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture as fixture};

pub(crate) use {worth_testkit_server::shared_support_product_operation_phase_eleven_backend::stateful_editor_registration, worth_testkit_server::shared_support_product_operation_phase_eleven_backend::StatefulProductEditorBackend};
pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload_with_title, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server, worth_testkit_server::shared_support_product_session_phase_ten_fixture::direct_session, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_product_mutation_request_with_basis_and_header, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_session_request};

pub(crate) fn base_digest(value: &str) -> WorthServerProductOperationBaseDigest {
    worth_server::WorthServerProductOperationBaseDigest::new(value).expect("valid base digest")
}