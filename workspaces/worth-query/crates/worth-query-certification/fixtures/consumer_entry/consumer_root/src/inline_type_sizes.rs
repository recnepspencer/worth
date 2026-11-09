//! Exact public layouts of this application's real operation and history owners.
use crate::application_program::{
    discovered_delivery::{Action, QueryDelivery},
    document_history::{HistoryAction, HistoryProgram, HistoryRoot},
    ConsumerFixedAction,
};
use crate::{ConsumerDiscoveredProgramRoot, ConsumerProgram, ConsumerProgramRoot, ConsumerSchema};
use worth_query_host::facade::application_entry::*;
use worth_query_topology_entry::{
    PlanarDocumentHistory, PlanarInitialAdjustment, PlanarSourceAdjustment,
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
type Edit = WorthQueryHistoryEdit<
    'static,
    ConsumerSchema,
    PlanarDocumentHistory<ConsumerSchema>,
    PlanarInitialAdjustment,
    HistoryProgram,
    HistoryAction,
>;
type Finish = WorthQueryHistoryEditFinish<
    'static,
    ConsumerSchema,
    PlanarDocumentHistory<ConsumerSchema>,
    PlanarInitialAdjustment,
    HistoryProgram,
    HistoryAction,
>;
type Group = WorthQueryGroupedHistoryDocument<
    'static,
    ConsumerSchema,
    PlanarDocumentHistory<ConsumerSchema>,
    HistoryProgram,
    WorthQueryHistoryTypedMembers<
        ConsumerSchema,
        PlanarInitialAdjustment,
        HistoryProgram,
        HistoryAction,
    >,
>;
type Execution = WorthQueryHistoryGroupExecution<
    'static,
    ConsumerSchema,
    PlanarDocumentHistory<ConsumerSchema>,
    HistoryProgram,
    WorthQueryHistoryTypedMembers<
        ConsumerSchema,
        PlanarInitialAdjustment,
        HistoryProgram,
        HistoryAction,
    >,
>;
type FixedOutcome = WorthQueryFixedHistoryEditOutcome<
    ConsumerSchema,
    PlanarInitialAdjustment,
    HistoryProgram,
    HistoryRoot,
    HistoryAction,
>;

#[test]
fn application_carriers_stay_within_declared_inline_ceilings() {
    let rows = [
        row::<Fixed>(),
        row::<Discovered>(),
        row::<Edit>(),
        row::<Finish>(),
        row::<Group>(),
        row::<Execution>(),
        row::<FixedOutcome>(),
        row::<
            WorthQueryRequiredCorrectionPreparation<
                'static,
                ConsumerSchema,
                PlanarSourceAdjustmentBinding<ConsumerSchema>,
                ConsumerProgram,
                ConsumerProgramRoot,
            >,
        >(),
        row::<WorthQueryHistoryEditAdvance<Edit>>(),
        row::<Result<Finish, (Edit, WorthQueryHistoryEditDenial)>>(),
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
