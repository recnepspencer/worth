use super::*;
use crate::application_program::historical::{FixedAction, FixedProgram, Original};
use worth_query_decl::facade::application_program::ApplicationActionDelivery;

#[test]
fn history_accepts_the_leaf_and_refuses_an_unqualified_descendant_graph_at_compile_time() {
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

    // A second applicable impl makes this expression ambiguous if a descendant
    // graph ever acquires history availability without its own completion proof.
    trait Ambiguous<A> {
        fn marker() {}
    }
    impl<T: ?Sized> Ambiguous<()> for T {}
    struct Supported;
    impl<
            T: ?Sized
                + WorthQueryHistoryOperationMode<ConsumerSchema, Original, FixedProgram, FixedAction>,
        > Ambiguous<Supported> for T
    {
    }
    type FullMode =
        <crate::application_program::historical::FixedDelivery as ApplicationActionDelivery<
            ConsumerSchema,
            worth_query_topology_entry::PlanarInitialAdjustmentBinding<ConsumerSchema>,
        >>::Mode;
    let _ = <FullMode as Ambiguous<_>>::marker;
}
