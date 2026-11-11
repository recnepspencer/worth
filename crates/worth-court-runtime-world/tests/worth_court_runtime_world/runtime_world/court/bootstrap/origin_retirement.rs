//! Retirement admission crosses the actual public World close gates.
use super::*;

#[test]
fn checkpoint_origin_retirement_requires_closed_world_and_an_actual_home_binding() {
    use WorldCheckpointOriginRetirementDenial as Denial;
    let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
    let owner = &court.world;
    let policy = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&policy);
    assert_eq!(
        owner
            .retire_checkpoint_origin(&request, &court.records.runtime)
            .unwrap_err(),
        Denial::OwnerNotClosed
    );
    let close = owner.try_prepare_close_admission(&request).unwrap();
    assert_eq!(
        owner
            .retire_checkpoint_origin(&request, &court.records.runtime)
            .unwrap_err(),
        Denial::Busy
    );
    drop(close);
    drop(owner.lifecycle_port().close(&request, owner).unwrap());
    assert_eq!(
        owner
            .retire_checkpoint_origin(&request, &court.records.runtime)
            .unwrap_err(),
        Denial::NotHomeBound
    );
}
