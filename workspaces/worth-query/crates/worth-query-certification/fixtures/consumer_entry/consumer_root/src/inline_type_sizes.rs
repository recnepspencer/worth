//! Exact public layouts of this application's real operation owners.
use crate::application_program::{
    discovered_delivery::{Action, QueryDelivery},
    ConsumerFixedAction,
};
use crate::{ConsumerDiscoveredProgramRoot, ConsumerProgram, ConsumerProgramRoot, ConsumerSchema};
use worth_query_host::facade::application_entry::*;
use worth_query_topology_entry::{
    PlanarSourceAdjustment,
    PlanarSourceAdjustmentBinding,
};

type Fixed = WorthQueryFixedOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    ConsumerProgram,
    ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type Discovered = WorthQueryDiscoveredOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    ConsumerProgram,
    ConsumerDiscoveredProgramRoot,
    Action<QueryDelivery>,
>;
#[test]
fn application_carriers_stay_within_declared_inline_ceilings() {
    let rows = [
        row::<Fixed>(),
        row::<Discovered>(),
        row::<
            WorthQueryRequiredCorrectionPreparation<
                'static,
                ConsumerSchema,
                PlanarSourceAdjustmentBinding<ConsumerSchema>,
                ConsumerProgram,
                ConsumerProgramRoot,
            >,
        >(),
    ];
    for (name, size) in &rows {
        println!("APPLICATION_TYPE_SIZE|{name}|{size}|{}", 128 * 1024);
    }
    let exceeded: Vec<_> = rows.iter().filter(|(_, size)| *size > 128 * 1024).collect();
    assert!(
        exceeded.is_empty(),
        "inline carrier ceiling exceeded: {exceeded:?}"
    );
}

fn row<T>() -> (&'static str, usize) {
    (std::any::type_name::<T>(), std::mem::size_of::<T>())
}
