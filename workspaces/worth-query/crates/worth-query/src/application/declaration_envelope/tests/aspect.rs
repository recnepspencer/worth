use crate::application::{test_declaration_aspect_key, WorthQueryDeclarationEnvelopeInput};
use crate::target_binding::WorthQueryBindingTargetWitness;

use super::support::{admitted_handle, progressed, AspectRichEnvelopeFamily, EnvelopeInput};

#[test]
fn envelopes_publish_scoped_crossing_aspects_without_widening_beyond_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");
    let receipt = handle
        .receipt_routes_from_progressed(progressed(
            &handle,
            EnvelopeInput::<AspectRichEnvelopeFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("aspect-rich receipt should issue"));
    let envelope = handle
        .envelope_routes(WorthQueryDeclarationEnvelopeInput::issued(receipt))
        .unwrap_or_else(|_| panic!("aspect-rich envelope should issue"));

    assert_eq!(
        envelope.aspect_contract(),
        envelope.receipt().aspect_contract()
    );
    assert_eq!(
        envelope.aspect_publication(),
        envelope.receipt().aspect_publication()
    );
    assert!(!envelope
        .aspect_publication()
        .present()
        .contains(&test_declaration_aspect_key("selection.material_edit")));
    assert!(envelope
        .aspect_publication()
        .masked()
        .contains(&test_declaration_aspect_key("selection.private_authority")));
}

#[test]
fn envelope_binding_target_retains_public_aspect_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            AspectRichEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("aspect-rich envelope should issue"));

    let binding = envelope.binding_target();
    let semantics = binding.erased_target().semantics();
    let (_, _, _, _, contract, publication) = semantics
        .declaration_envelope()
        .expect("envelope binding target should retain public aspect semantics");

    assert_eq!(contract, envelope.aspect_contract());
    assert_eq!(publication, envelope.aspect_publication());
}

#[test]
fn envelope_digest_changes_when_public_aspect_publication_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let aspectful = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            AspectRichEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("aspect-rich envelope should issue"));
    let plain = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            super::support::RelationalEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("plain envelope should issue"));

    assert_ne!(aspectful.envelope_digest(), plain.envelope_digest());
}
