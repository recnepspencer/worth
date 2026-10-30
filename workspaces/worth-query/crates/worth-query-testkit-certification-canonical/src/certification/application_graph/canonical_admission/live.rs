//! Real retained-effect delivery keeps admitted nested values through its receipt and ACK.
#[path = "live/cause.rs"]
pub mod cause;
#[path = "live/native.rs"]
pub mod native;
#[path = "live/publish.rs"]
pub mod publish;
#[path = "live/query.rs"]
pub mod query;

pub use super::{host, opaque, schema::AdmissionSchema};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    primary_graph::*,
};

pub type Subscription<'host> =
    WorthQueryApplicationLiveSubscription<'host, AdmissionSchema, query::ReadLive>;

pub fn open(host: &host::Host) -> Subscription<'_> {
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    host.program
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.program.current_world())
        .query(query::ReadLive {
            identity: "root".to_owned(),
        })
        .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1))
        .expect("ordinary installed nested live query opens without projecting a value")
}

pub fn publish(host: &host::Host, idempotency: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    let outcome = host
        .program
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.program.current_world())
        .advanced()
        .mutate(publish::Publish {
            changed: cause::Changed {
                identity: "root".to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(
            &host.program,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the ordinary installed effect-only mutation executes");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "only a genuine native publication may supply the live cause"
    );
}

thread_local! {
    static RETURNED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static CANCEL_AFTER_PROJECTION: std::cell::RefCell<Option<WorthQueryCancellationSource>> = const { std::cell::RefCell::new(None) };
}
pub fn reset() {
    RETURNED.set(0);
    opaque::reset();
}
pub fn returned() -> u32 {
    RETURNED.get()
}
pub fn projector_returned() -> bool {
    RETURNED.set(RETURNED.get() + 1);
    CANCEL_AFTER_PROJECTION.with_borrow(|source| {
        if let Some(source) = source {
            source.cancel();
            true
        } else {
            false
        }
    })
}
pub struct CancelAfterProjection;
impl CancelAfterProjection {
    pub fn arm(source: WorthQueryCancellationSource) -> Self {
        reset();
        CANCEL_AFTER_PROJECTION.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some(source);
        });
        Self
    }
}
impl Drop for CancelAfterProjection {
    fn drop(&mut self) {
        CANCEL_AFTER_PROJECTION.with_borrow_mut(|slot| *slot = None);
    }
}
