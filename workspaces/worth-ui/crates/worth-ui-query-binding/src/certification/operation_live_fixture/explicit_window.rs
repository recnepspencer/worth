use worth_query::facade::{domain, installed};

use crate::{WorthUiCollectionAllocationPolicy, WorthUiOperationLiveResource};

use super::{collection_breadth, operation_live_requirements, WorthUiOperationLiveTestFixture};

impl WorthUiOperationLiveTestFixture {
    pub fn open_tail_resource(&mut self) -> WorthUiOperationLiveResource {
        let first = crate::operation_live::settle_once(
            &self.reference,
            operation_live_requirements(),
            &mut self.workspace,
        )
        .expect("tail fixture first settlement");
        let collection = match installed::transition::collection_capability(
            first.settled.into_bound_collection(),
        ) {
            installed::transition::WorthQueryCollectionCapabilityTransition::Bound(value) => value,
            installed::transition::WorthQueryCollectionCapabilityTransition::Denied(_)
            | installed::transition::WorthQueryCollectionCapabilityTransition::Stale(_) => {
                panic!("tail fixture collection capability stopped")
            }
        };
        let breadth = collection_breadth(self.breadth);
        let beginning =
            admit_window(collection.declare_window(collection.beginning_cursor(), breadth));
        let beginning = resolve_window(collection.resolve_window(beginning));
        let cursor = match beginning.continuation() {
            domain::WorthQueryCollectionContinuation::LiveMore(cursor) => cursor.clone(),
            _ => panic!("tail fixture requires rows beyond the beginning window"),
        };
        let tail = admit_window(collection.declare_window(cursor, breadth));
        let tail = resolve_window(collection.resolve_window(tail));
        let consumer = domain::WorthQueryCollectionConsumerWindow::from_bound(collection, tail)
            .expect("tail fixture consumer");
        let live = crate::operation_live::settle_once(
            &self.reference,
            operation_live_requirements(),
            &mut self.workspace,
        )
        .expect("tail fixture live settlement");
        self.workspace
            .with_advancement_mut(|phase, workspace| {
                WorthUiOperationLiveResource::open_with_consumer(
                    &phase,
                    crate::operation_live::WorthUiOperationLiveSources {
                        installed_reference: self.reference.clone(),
                        settlement: live,
                    },
                    consumer,
                    WorthUiCollectionAllocationPolicy::PreserveAdmittedRows,
                    workspace,
                )
                .expect("tail fixture live resource")
            })
            .expect("workspace host call opens")
    }
}

fn admit_window(
    outcome: domain::WorthQueryCollectionWindowAdmissionOutcome,
) -> domain::WorthQueryAdmittedCollectionWindow {
    match installed::transition::collection_window_admission(outcome) {
        installed::transition::WorthQueryCollectionWindowTransition::Admitted(value) => value,
        installed::transition::WorthQueryCollectionWindowTransition::Denied(_)
        | installed::transition::WorthQueryCollectionWindowTransition::Stale(_)
        | installed::transition::WorthQueryCollectionWindowTransition::RebindRequired(_) => {
            panic!("tail fixture window admission stopped")
        }
    }
}

fn resolve_window(
    outcome: domain::WorthQueryCollectionWindowOutcome,
) -> domain::WorthQueryBoundCollectionWindow {
    match installed::transition::collection_window_resolution(outcome) {
        installed::transition::WorthQueryCollectionWindowTransition::Admitted(value) => value,
        installed::transition::WorthQueryCollectionWindowTransition::Denied(_)
        | installed::transition::WorthQueryCollectionWindowTransition::Stale(_)
        | installed::transition::WorthQueryCollectionWindowTransition::RebindRequired(_) => {
            panic!("tail fixture window resolution stopped")
        }
    }
}
