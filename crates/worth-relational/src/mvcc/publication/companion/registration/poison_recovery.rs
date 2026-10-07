//! A fault at each registry door leaves typed state usable by the next publisher.
use super::super::{
    CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
};
use super::guard_fault::{self, Door};
use super::*;
use crate::tests::support::*;

#[derive(Debug)]
struct Unindexed;
impl RelationalPublicationCompanion for Unindexed {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        context.seal_replacement(reserved, Arc::new(1_u64))
    }
}
fn budget() -> CompanionPreflightBudget {
    CompanionPreflightBudget {
        maximum_work_visits: 32,
        maximum_preparation_bytes: 8192,
    }
}
fn catch(run: impl FnOnce()) {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).is_err());
}
fn recover(door: Door) {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "registry-poison-before");
    let port = runtime.publication_companion_port();
    let registry = port.publication.companion_registry();
    let pending = port.begin_required_registration().unwrap();
    if door == Door::Activate {
        guard_fault::arm(door);
        catch(|| {
            pending.activate(Arc::new(Unindexed), budget()).unwrap();
        });
        // A consumed pending registration is not reconstructed. The owner
        // legitimately begins a new generation under the recovered guard.
        let _registration = port
            .begin_required_registration()
            .unwrap()
            .activate(Arc::new(Unindexed), budget())
            .unwrap();
        let committed = create_entity_outcome(&runtime, "registry-poison-after-activate");
        release_test_commit_snapshot(&runtime, &committed);
        return;
    }
    let mut registration = pending.activate(Arc::new(Unindexed), budget()).unwrap();
    guard_fault::arm(door);
    catch(|| match door {
        Door::Fork => {
            registry.fork();
        }
        Door::Enter => {
            registry.enter().unwrap();
        }
        Door::Begin => {
            port.begin_required_registration().unwrap();
        }
        Door::Remove => {
            port.remove_required(&registration).unwrap();
        }
        Door::Head => {
            registration
                .with_branch_cell_at_head(
                    &runtime,
                    &runtime.main_branch_identity(),
                    Arc::new(0_u64),
                    |_| (),
                )
                .unwrap();
        }
        Door::Activate => unreachable!(),
    });
    // Rust read guards do not poison a lock on unwind. Give both read doors
    // an actual poisoned write epoch too, so their recovery is exercised.
    if matches!(door, Door::Fork | Door::Enter) {
        catch(|| {
            let _guard = registry.state.write().unwrap();
            panic!("poison the registry write epoch");
        });
    }
    assert!(registry.state.is_poisoned());
    match door {
        Door::Fork => {
            assert!(matches!(
                registry.fork().enter().unwrap().active(),
                Err(PublicationCompanionRegistrationStop::RebindRequired)
            ));
        }
        Door::Enter => {
            assert!(registry.enter().unwrap().active().unwrap().is_some());
        }
        Door::Begin => {
            registration = port
                .begin_required_registration()
                .unwrap()
                .activate(Arc::new(Unindexed), budget())
                .unwrap();
        }
        Door::Remove => {
            port.remove_required(&registration).unwrap();
            registration = port
                .begin_required_registration()
                .unwrap()
                .activate(Arc::new(Unindexed), budget())
                .unwrap();
        }
        Door::Head => {
            registration
                .with_branch_cell_at_head(
                    &runtime,
                    &runtime.main_branch_identity(),
                    Arc::new(0_u64),
                    |_| (),
                )
                .unwrap();
        }
        Door::Activate => unreachable!(),
    }
    let committed = create_entity_outcome(&runtime, "registry-poison-after");
    release_test_commit_snapshot(&runtime, &committed);
    assert!(registry.enter().unwrap().active().unwrap().is_some());
    drop(registration);
}
#[test]
fn fork_recovers_registry_poison() {
    recover(Door::Fork);
}
#[test]
fn enter_recovers_registry_poison() {
    recover(Door::Enter);
}
#[test]
fn begin_recovers_registry_poison() {
    recover(Door::Begin);
}
#[test]
fn remove_recovers_registry_poison() {
    recover(Door::Remove);
}
#[test]
fn activate_recovers_registry_poison() {
    recover(Door::Activate);
}
#[test]
fn head_selection_recovers_registry_poison() {
    recover(Door::Head);
}
