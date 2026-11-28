use super::RuntimeServices;
use crate::runtime::state::subsystems::RuntimeSubsystem;

#[test]
fn runtime_services_fork_issues_an_independent_sequence() {
    let services = <RuntimeServices as RuntimeSubsystem>::new(&());
    let first_tx = services.next_transaction_id();
    let first_runtime_id = services.runtime_instance_id();

    let forked = RuntimeSubsystem::fork(&services);

    assert_eq!(first_tx.0, 1);
    assert_ne!(forked.runtime_instance_id(), first_runtime_id);
    assert_eq!(forked.next_transaction_id().0, 1);
}
