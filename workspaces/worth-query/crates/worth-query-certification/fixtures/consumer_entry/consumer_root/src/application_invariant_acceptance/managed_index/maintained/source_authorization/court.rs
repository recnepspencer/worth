//! Source-only authorization changes have no Parent permission check to mask them.
use super::super::super::super::{authentication, managed_index::lifecycle};
use super::super::{authoring::Edit, host, mutation, protected::status};
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, runtime::ExecutionAllocationPolicy,
};
mod correction;
mod expectations;
mod reads;

struct Setup {
    world: host::Host,
    calls: Configuration,
    scope: WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedExternalPrincipal<Catalog>,
}
impl Setup {
    fn new() -> Self {
        let calls = Configuration::default();
        let world = host::install_source_authorization(calls.clone());
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let result = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world())
            .advanced()
            .mutate(Edit::Item {
                directory: "catalog".into(),
                identity: "authorized-item".into(),
                name: "shared".into(),
            })
            .idempotency(&0x919c_0000u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        Self {
            world,
            calls,
            scope,
            principal,
        }
    }
    fn set_readable(&self, readable: bool, key: u64) {
        let before = self.calls.native.updates.load(Ordering::SeqCst);
        let result = self
            .world
            .runtime()
            .request(&self.principal, &self.scope)
            .on_branch(self.world.current_world())
            .advanced()
            .mutate(status::SetStatus {
                identity: "watch-access".into(),
                active: true,
                readable,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&self.world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert_eq!(
            self.calls.native.updates.load(Ordering::SeqCst),
            before,
            "only Source READ policy changed; the Native Source was not edited"
        );
    }
    fn rename() -> mutation::Rename {
        mutation::Rename {
            directory: "catalog".into(),
            prefix: "sh".into(),
            item: "authorized-item".into(),
            replacement: "free".into(),
        }
    }
}
fn native() -> FindNative {
    FindNative {
        directory: "catalog".into(),
        prefix: "sh".into(),
    }
}
fn completed() -> FindCompleted {
    FindCompleted {
        directory: "catalog".into(),
        prefix: "sh".into(),
    }
}
fn callbacks(calls: &prefix::Calls) -> (usize, usize) {
    (
        calls.probe_validations.load(Ordering::SeqCst),
        calls.matched_sources.load(Ordering::SeqCst),
    )
}
