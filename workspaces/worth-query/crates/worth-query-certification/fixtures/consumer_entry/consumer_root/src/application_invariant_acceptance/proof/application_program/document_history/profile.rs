use super::*;
use crate::application_program::historical::{DiscoveredAction, DiscoveredProgram, Original};
use worth_query_decl::facade::application_program::ApplicationActionDelivery;

#[test]
fn history_accepts_fixed_graphs_and_refuses_discovered_delivery_at_compile_time() {
    fn supported<
        T: WorthQueryHistoryOperationMode<
            ConsumerSchema,
            PlanarInitialAdjustment,
            HistoryProgram,
            HistoryAction,
        >,
    >() {
    }
    supported::<<crate::application_program::document_history::HistoryDelivery as ApplicationActionDelivery<ConsumerSchema,
        worth_query_topology_entry::PlanarInitialAdjustmentBinding<ConsumerSchema>>>::Mode>();

    fn full<
        T: WorthQueryHistoryOperationMode<
            ConsumerSchema,
            Original,
            crate::application_program::historical::FixedProgram,
            crate::application_program::historical::FixedAction,
        >,
    >() {
    }
    full::<
        <crate::application_program::historical::FixedDelivery as ApplicationActionDelivery<
            ConsumerSchema,
            worth_query_topology_entry::PlanarInitialAdjustmentBinding<ConsumerSchema>,
        >>::Mode,
    >();
    // A second applicable impl makes this expression ambiguous if discovered
    // delivery acquires availability without its distinct completion contract.
    trait Ambiguous<A> {
        fn marker() {}
    }
    impl<T: ?Sized> Ambiguous<()> for T {}
    struct Supported;
    impl<
            T: ?Sized
                + WorthQueryHistoryOperationMode<
                    ConsumerSchema,
                    Original,
                    DiscoveredProgram,
                    DiscoveredAction,
                >,
        > Ambiguous<Supported> for T
    {
    }
    type UnsupportedMode =
        <crate::application_program::historical::DiscoveredDelivery as ApplicationActionDelivery<
            ConsumerSchema,
            worth_query_topology_entry::PlanarInitialAdjustmentBinding<ConsumerSchema>,
        >>::Mode;
    let _ = <UnsupportedMode as Ambiguous<_>>::marker;
}
