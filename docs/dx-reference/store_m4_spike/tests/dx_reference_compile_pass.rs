#[path = "../../store_m4_scale_and_branches.rs"]
mod store_m4_scale_and_branches;

use std::sync::atomic::{AtomicU64, Ordering};
use store_m4_dx_spike::*;

struct StubRetentionPort {
    next_id: AtomicU64,
}

impl RuntimeWorldRetentionPort for StubRetentionPort {
    fn capture_and_pin_frontier(&self, _spec: PinSpec) -> Result<PinnedFrontierLease, AppError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        Ok(PinnedFrontierLease::demo(id))
    }

    fn expire_lease(&self, _pin_id: PinId) -> Result<LeaseTransition, AppError> {
        Ok(LeaseTransition::new(LeaseState::Expiring))
    }
}

#[test]
fn dx_reference_scale_and_branches_compiles_and_executes_seam() {
    let principal = Principal;
    let scope = AccessScope;
    store_m4_scale_and_branches::sustained_scale_and_branching_journey(
        std::path::Path::new("stub-home"),
        &principal,
        &scope,
    )
    .expect("M4 DX journey compiles and succeeds against stub contract");

    let home: Home<MyApp> = Home::directory(std::path::Path::new("stub-home"))
        .open()
        .expect("open stub home");
    let req: Request<'_, MySchema> = home.request(&principal, &scope).expect("stub request");
    let port = StubRetentionPort {
        next_id: AtomicU64::new(1),
    };
    let disposition = store_m4_scale_and_branches::semantic_git_nested_fork_pin_and_retry_seam(
        &port,
        PinScope::new(1),
        PinScope::new(2),
        &req,
    )
    .expect("owner-to-owner pin seam succeeds");
    assert_eq!(disposition, Disposition::Complete);
}
