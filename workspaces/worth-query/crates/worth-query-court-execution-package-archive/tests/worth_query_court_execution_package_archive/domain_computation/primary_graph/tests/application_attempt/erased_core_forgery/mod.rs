// The process part of this module defines `admit!` too; macros are textually scoped, so the
// courts mounted here carry their own copy.
macro_rules! admit {
    ($world:expr, $principal:expr, $account:expr, $operation:ty, $request:expr) => {{
        let operation = $world
            .application
            .installed_schema()
            .installed_operation(<$operation>::reference())
            .unwrap();
        $world
            .application
            .with_host_advancement(|phase| {
                $world
                    .selected_product(&(phase).test_host_request_for_test())
                    .authorize_operation(
                        &phase,
                        $principal,
                        $account,
                        &operation,
                        Default::default(),
                        $request,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call")
    }};
}
#[path = "commit_entries/mod.rs"]
pub mod commit_entries;
#[path = "decision_entry/mod.rs"]
pub mod decision_entry;
#[path = "installed_seal/mod.rs"]
pub mod installed_seal;
#[path = "precedence/mod.rs"]
pub mod precedence;
#[path = "recovery_entries/mod.rs"]
pub mod recovery_entries;
#[path = "workflow_entry/mod.rs"]
pub mod workflow_entry;
