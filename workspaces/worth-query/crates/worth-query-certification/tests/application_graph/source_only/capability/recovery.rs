//! Fresh binding-selected capability admission protects retained native source custody.
use super::{host, program::*, schema::*};
use crate::document_retention_model::operator_identity::request_scope;
use worth_query_host::facade::{
    application_entry::*, declaration::application_schema::*, primary_graph::*,
};
#[test]
fn capability_partial_requires_fresh_capability_admission_without_source_rerun() {
    let host = host::install();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);

    let request = host.application.runtime().request(&actor, &scope);
    // Establish a genuine committed World basis; bootstrap has no commit identity.
    let warm = request
        .mutate(Original {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(0x91765008_u64)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program, Action>(&host.application)
        .unwrap_or_else(|(_, cause)| panic!("warm source constructs: {cause:?}"));
    let warm = match warm.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(warm) => warm,
        _ => panic!("the prior production journey establishes a real World commit"),
    };
    warm.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("warm close: {cause:?}"));
    let owner = request
        .mutate(Original {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 18,
            },
        })
        .without_source()
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            DocumentIdentityField::reference(),
            ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned()).unwrap(),
        ))
        .idempotency_owned(0x91765018_u64)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program, Action>(&host.application)
        .unwrap_or_else(|(_, cause)| panic!("source constructs: {cause:?}"));
    let policy = crate::test_request_policy::serial_request();
    host.application
        .runtime()
        .fail_next_durable_append_for_test(&worth_execution::ExecutionRequest::serial(&policy));
    let owner = match owner.advance(&request) {
        WorthQuerySourceOnlyAdvance::Progress(owner) => owner,
        _ => panic!("real append fault must retain original source"),
    };
    assert_eq!(
        owner.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(host.contacts.counts(), (2, 2));
    let before = host
        .application
        .runtime()
        .with_host_advancement(|phase| {
            request
                .retain_read(&phase)
                .unwrap()
                .selected_commit()
                .clone()
        })
        .unwrap();
    assert_eq!(super::readback::grant_status(&host), 1);
    drop(request);
    drop(actor);
    drop(scope);
    let refused_scope = request_scope();
    let revoke_only = host::authenticate(&host, 2, &refused_scope);
    let refused = host
        .application
        .runtime()
        .request(&revoke_only, &refused_scope);
    let owner = match owner.advance(&refused) {
        WorthQuerySourceOnlyAdvance::Refused {
            operation,
            denial:
                WorthQuerySourceOnlyDenial::Recovery(
                    WorthQueryApplicationRecoveryRequestDenial::Request(
                        WorthQueryApplicationRequestMutationDenial::Authorization(denial),
                    ),
                ),
        } => {
            assert_eq!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing,
                )
            );
            operation
        }
        _ => panic!("revoke-only actor must fail native retention capability admission"),
    };
    assert_eq!(
        owner.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(host.contacts.counts(), (2, 2));
    assert_eq!(
        host.application
            .runtime()
            .with_host_advancement(|phase| refused
                .retain_read(&phase)
                .unwrap()
                .selected_commit()
                .clone())
            .unwrap(),
        before
    );
    assert_eq!(
        host.application
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    drop(refused);
    drop(revoke_only);
    drop(refused_scope);
    let fresh_scope = request_scope();
    let fresh_actor = host::authenticate(&host, 1, &fresh_scope);
    let fresh = host
        .application
        .runtime()
        .request(&fresh_actor, &fresh_scope);
    let owner = match owner.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Progress(owner) => owner,
        _ => panic!("the original partial remains recoverable without rerunning its source"),
    };
    let completed = match owner.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        _ => panic!("native original keyed completion"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            publication: Some(_),
            ..
        }
    ));
    assert_eq!(
        fresh
            .query(super::readback::ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        18
    );
    assert_eq!(host.contacts.counts(), (2, 2));
    let catalog = host
        .application
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    let inspector = host
        .application
        .runtime()
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let (completed, denial) = completed
        .close(&fresh)
        .err()
        .expect("actual retained inspector refuses release");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Recovery(_)
    ));
    drop(held);
    drop(fresh);
    drop(fresh_actor);
    drop(fresh_scope);
    let close_scope = request_scope();
    let close_actor = host::authenticate(&host, 1, &close_scope);
    let close = host
        .application
        .runtime()
        .request(&close_actor, &close_scope);
    completed
        .close(&close)
        .unwrap_or_else(|(_, cause)| panic!("original cleanup retry: {cause:?}"));
    assert!(host
        .application
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
