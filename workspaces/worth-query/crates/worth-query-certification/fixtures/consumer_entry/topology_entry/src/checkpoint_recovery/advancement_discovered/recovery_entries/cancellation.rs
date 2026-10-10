//! Cancellation at the consumer input boundary reaches the real recovery reader.
use super::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use worth_query_decl::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};

struct CancelOnInput {
    edit: crate::PlanarEdit,
    cancellation: Arc<authentication::WorthQueryCancellationSource>,
    encoded: Arc<AtomicBool>,
}
impl ApplicationMutationIntent<CheckpointSchema> for CancelOnInput {
    type Binding = crate::planar_edit::PlanarEditBinding<CheckpointSchema>;
    fn input(&self) -> &crate::PlanarMutation {
        // Identities are encoded after authorize_recovery's liveness check,
        // before prepare_selected resolves the authenticated principal.
        self.encoded.store(true, Ordering::SeqCst);
        self.cancellation.cancel();
        &self.edit.0
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<CheckpointSchema>>::ScopeBinding {
        <crate::PlanarEdit as ApplicationMutationIntent<CheckpointSchema>>::scope_binding(
            &self.edit,
        )
    }
}

#[test]
fn cancellation_in_recovery_authorization_uses_the_opening_spelling() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let application = install(None);
        let (live_scope, principal) = authenticate(&application);
        let source = source!(application.request(&principal, &live_scope));
        let cancellation = Arc::new(authentication::WorthQueryCancellationSource::new());
        let scope = authentication::WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(30),
            cancellation.token(),
        );
        let encoded = Arc::new(AtomicBool::new(false));
        let intent = CancelOnInput {
            edit: crate::PlanarEdit(crate::PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                    worth_query_consumer_values::PlanarAdjustment {
                        body_key: "anchor-a".into(),
                        replacement_y: length(2),
                    },
                ]),
            }),
            cancellation: cancellation.clone(),
            encoded: encoded.clone(),
        };
        let mutation = application
            .request(&principal, &scope)
            .mutate(intent)
            .expect_source(source)
            .idempotency(&9140_u64);
        assert!(
            !encoded.load(Ordering::SeqCst),
            "building the request does not cancel it"
        );
        reports();
        let denial = mutation
            .resolve_idempotency_in_program(&application)
            .unwrap_err();
        assert!(
            encoded.load(Ordering::SeqCst),
            "the recovery helper passed liveness and encoded its input"
        );
        assert!(matches!(denial, RecoveryDenial::Recovery(NativeDenial::ExecutionDenied(Denial::Interrupted(
            worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
        )))), "the real principal reader uses the opening spelling: {denial:?}");
        assert_eq!(reports().len(), 1, "the cancelled call owns one request");
    }
}
