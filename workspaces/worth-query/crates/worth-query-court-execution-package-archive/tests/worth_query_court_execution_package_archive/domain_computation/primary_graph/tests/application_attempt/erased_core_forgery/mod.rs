// The process part of this module defines `admit!` too; macros are textually scoped, so the
// precedence court mounted here carries its own copy.
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
#[path = "decision_entry/mod.rs"]
pub mod decision_entry;
#[path = "precedence/mod.rs"]
pub mod precedence;
