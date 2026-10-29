use worth_relational::facade::branch::RelationalForkPort;
use worth_relational::facade::mvcc::{
    RelationalPreparationPort, RelationalPublicationPort, RelationalSettlementPort,
};
use worth_relational::facade::runtime::RelationalRuntime;

fn assert_clone_send_sync<T: Clone + Send + Sync>() {}

fn obtain_independent_ports(
    runtime: &RelationalRuntime,
) -> (
    RelationalPreparationPort,
    RelationalForkPort,
    RelationalPublicationPort,
    RelationalSettlementPort,
) {
    (
        runtime.preparation_port(),
        runtime.fork_port(),
        runtime.publication_port(),
        runtime.settlement_port(),
    )
}

fn main() {
    assert_clone_send_sync::<RelationalPreparationPort>();
    assert_clone_send_sync::<RelationalForkPort>();
    assert_clone_send_sync::<RelationalPublicationPort>();
    assert_clone_send_sync::<RelationalSettlementPort>();
    let _ = obtain_independent_ports;
}
