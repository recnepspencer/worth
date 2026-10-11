//! Seeded schedules cover queued retirement, immediate readers and both lost-cue orders.
use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
pub(super) fn run(context: &world::Context, seed: u64, divergences: &mut Vec<String>) {
    let mut world = World::new(context, seed);
    let mut step = 0;
    let mut apply = |world: &mut World<'_>, label: &str, action: &mut dyn FnMut(&mut World<'_>)| {
        step += 1;
        let result = catch_unwind(AssertUnwindSafe(|| action(world)));
        if result.is_err() {
            divergences.push(format!(
                "seed {seed}, step {step}, {label}: production panic"
            ));
            // A caught dangling-note panic poisoned the mutex. Do not let teardown
            // counterfeit another failure by panicking inside an owner's Drop.
            world
                .registry
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .closed_cleanup = Default::default();
        }
        if let Some(failure) = world.failure.take() {
            divergences.push(format!("seed {seed}, step {step}, {label}: {failure}"));
        }
        if let Some(key) = observe::dangling(world) {
            divergences.push(format!(
                "seed {seed}, step {step}, {label}: dangling note {}",
                key.producer
            ));
            world
                .registry
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .closed_cleanup = Default::default();
            return true;
        }
        observe::compare(world, seed, step, label, divergences, false);
        false
    };
    apply(&mut world, "initial", &mut |_| {});
    if let Some(id) = world.extra_dependent {
        apply(&mut world, "second dependent owner drop", &mut |w| {
            w.drop_owner(id)
        });
    }
    let case = seed % 4;
    if case == 0 {
        apply(&mut world, "A dependent owner drop", &mut |w| {
            w.drop_owner(2)
        });
        if apply(
            &mut world,
            "A queued product occurrence retirement",
            &mut |w| {
                w.registry.release_product_occurrence(occurrence());
                w.model.retire_product(1);
            },
        ) {
            return;
        }
        apply(&mut world, "A affording admit", &mut |w| {
            w.admission(1_000_000).unwrap();
        });
        apply(&mut world, "A newest owner Drop after admit", &mut |w| {
            w.drop_owner(1)
        });
    } else if case == 1 {
        apply(&mut world, "B newest owner drop", &mut |w| w.drop_owner(1));
        apply(&mut world, "B dependent owner drop", &mut |w| {
            w.drop_owner(2)
        });
        apply(&mut world, "B admission-free ensure_bound", &mut |w| {
            w.bind()
        });
    } else {
        // C1: claimant closes and its old cue is consumed before supersession.
        // C2: a published successor replaces a closed claimed predecessor.
        if case == 3 {
            apply(
                &mut world,
                "C cached claimant drop before supersession",
                &mut |w| w.drop_owner(2),
            );
            apply(&mut world, "C admit before supersession", &mut |w| {
                w.admission(1_000_000).unwrap();
            });
        }
        apply(
            &mut world,
            "C supersede published claimed predecessor",
            &mut |w| {
                supersede_predecessors(
                    &mut w.registry.state.lock().unwrap(),
                    &w.keys[1],
                    &mut record_admission(),
                )
                .unwrap();
                w.model.supersede(1);
            },
        );
        apply(&mut world, "C successor publishes Ready", &mut |w| {
            let DemandState::Output(output) = &context.ready else {
                panic!("Ready")
            };
            let Some(super::super::super::WorthQueryOutputCheckpoint::Ready(ready)) =
                &output.checkpoint
            else {
                panic!("Ready")
            };
            let mut state = w.registry.state.lock().unwrap();
            state.records.get_mut(&w.keys[1]).unwrap().state =
                DemandState::Output(super::super::super::WorthQueryOutputProgress::new(
                    super::super::super::WorthQueryOutputCheckpoint::Ready(ready.clone()),
                ));
            w.model.rows.get_mut(&1).unwrap().phase = Phase::Ready;
            drop(state);
            w.model.settle();
        });
        if case == 2 {
            apply(&mut world, "C dependent owner drop", &mut |w| {
                w.drop_owner(2)
            });
        }
        apply(&mut world, "C newest owner drop", &mut |w| w.drop_owner(1));
    }
    if seed & 8 != 0 {
        apply(&mut world, "old owner drop", &mut |w| w.drop_owner(0));
    }
    if seed & 4 != 0 {
        apply(&mut world, "prepared token release", &mut |w| {
            w.registry.release_prepared_token(&context.commit(1));
            if let Some(entry) = w.model.custody.get_mut(&1) {
                entry.tokens -= 1;
            }
            w.model.settle();
        });
    }
    let own_work = World::own_work(context, world.generation);
    apply(
        &mut world,
        "admit exactly its own work",
        &mut |w| match w.admission(own_work) {
            Err(denial) => {
                w.failure = Some(format!(
                    "foreign debt refused own work: {:?}",
                    denial.kind()
                ))
            }
            Ok(charged) if charged != own_work => {
                w.failure = Some(format!("foreign quote charged {charged}, own {own_work}"))
            }
            Ok(_) => {}
        },
    );
    apply(&mut world, "one-short admit", &mut |w| {
        let denial = w.admission(own_work - 1).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
    });
    for visit in 0..4 {
        apply(
            &mut world,
            &format!("quiescent admission {visit}"),
            &mut |w| {
                w.admission(1_000_000).unwrap();
            },
        );
    }
    apply(
        &mut world,
        "admission-free binding after quiescence",
        &mut |w| w.bind(),
    );
    apply(&mut world, "unused program source owner closes", &mut |w| {
        w.registry.discard_prepared_source(&context.commit(0));
        w.model.custody.remove(&0);
    });
    observe::compare(&world, seed, step + 1, "quiescent value", divergences, true);
}
