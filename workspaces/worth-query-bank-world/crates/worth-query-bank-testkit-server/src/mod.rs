//! One integration executable; scenario support is shared by its cost-class courts.
#[allow(dead_code, unused_imports)]
#[path = "activity_history.rs"]
pub mod activity_history;
#[allow(dead_code, unused_imports)]
#[path = "approved_payment_workflow.rs"]
pub mod approved_payment_workflow;
#[allow(dead_code, unused_imports)]
#[path = "ordinary_mutations/authorization_time.rs"]
pub mod authorization_time;
#[allow(dead_code, unused_imports)]
#[path = "authorized_proposals/mod.rs"]
pub mod authorized_proposals;
#[allow(dead_code, unused_imports)]
#[path = "business_authorization.rs"]
pub mod business_authorization;
#[allow(dead_code, unused_imports)]
#[path = "ordinary_mutations/estate_operations.rs"]
pub mod estate_operations;
#[allow(
    dead_code,
    reason = "the shared read fixture includes scenarios used by sibling consumer courtrooms"
)]
#[allow(unused_imports)]
#[path = "ordinary_reads/fixture.rs"]
pub mod fixture;
#[allow(dead_code, unused_imports)]
#[path = "identity_admission.rs"]
pub mod identity_admission;
#[allow(dead_code, unused_imports)]
#[path = "identity_denials.rs"]
pub mod identity_denials;
#[allow(dead_code, unused_imports)]
#[path = "live_activity.rs"]
pub mod live_activity;
#[allow(dead_code, unused_imports)]
#[path = "ordinary_mutations.rs"]
pub mod ordinary_mutations;
#[allow(dead_code, unused_imports)]
#[path = "ordinary_reads/mod.rs"]
pub mod ordinary_reads;
#[allow(dead_code, unused_imports)]
#[path = "parallel_history/mod.rs"]
pub mod parallel_history;
#[allow(dead_code, unused_imports)]
#[path = "public_create_personal_account.rs"]
pub mod public_create_personal_account;
#[allow(dead_code, unused_imports)]
#[path = "support/mod.rs"]
pub mod support;
#[allow(dead_code, unused_imports)]
#[path = "workflow_continuations.rs"]
pub mod workflow_continuations;
pub use ordinary_mutations::*;
