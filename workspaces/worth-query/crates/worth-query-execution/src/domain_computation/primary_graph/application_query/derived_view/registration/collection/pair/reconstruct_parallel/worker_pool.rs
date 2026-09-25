//! Fixed-width scoped reads; only ordered results reach projection.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Mutex};

use worth_relational::facade::identity::EntityId;

use super::{Denial, MAX_PAIR_READS_IN_FLIGHT};

pub(super) fn read_bounded_pairs<Member, First, Second>(
    members: &[(EntityId, Member)],
    read_pair: &(impl Fn(&Member) -> Result<(First, Second), Denial> + Sync),
    mut accept: impl FnMut(usize, First, Second) -> Result<(), Denial>,
) -> Result<(), Denial>
where
    Member: Sync,
    First: Send,
    Second: Send,
{
    if members.is_empty() {
        return Ok(());
    }
    let width = members.len().min(MAX_PAIR_READS_IN_FLIGHT);
    let (task_sender, task_receiver) = mpsc::sync_channel::<usize>(width);
    let task_receiver = Mutex::new(task_receiver);
    std::thread::scope(|scope| {
        let (result_sender, result_receiver) =
            mpsc::sync_channel::<(usize, Result<(First, Second), Denial>)>(width);
        let mut workers = Vec::with_capacity(width);
        let mut spawn_failed = false;
        for _ in 0..width {
            let sender = result_sender.clone();
            let task_receiver = &task_receiver;
            match std::thread::Builder::new().spawn_scoped(scope, move || loop {
                let task = task_receiver
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .recv();
                let Ok(index) = task else { break };
                let result = catch_unwind(AssertUnwindSafe(|| read_pair(&members[index].1)))
                    .unwrap_or(Err(Denial::QueryExecutionDenied));
                let failed = result.is_err();
                if sender.send((index, result)).is_err() || failed {
                    break;
                }
            }) {
                Ok(worker) => workers.push(worker),
                Err(_) => {
                    spawn_failed = true;
                    break;
                }
            }
        }
        drop(result_sender);
        let result = if spawn_failed {
            Err(Denial::QueryExecutionDenied)
        } else {
            let mut dispatch_failed = false;
            for index in 0..width {
                if task_sender.send(index).is_err() {
                    dispatch_failed = true;
                    break;
                }
            }
            let mut next_dispatch = width;
            let mut next_accept = 0;
            let mut ready = BTreeMap::new();
            let mut result = if dispatch_failed {
                Err(Denial::QueryExecutionDenied)
            } else {
                Ok(())
            };
            while result.is_ok() && next_accept < members.len() {
                let received = result_receiver
                    .recv()
                    .map_err(|_| Denial::QueryExecutionDenied);
                let (index, observation) = match received {
                    Ok(value) => value,
                    Err(denial) => {
                        result = Err(denial);
                        break;
                    }
                };
                if ready.insert(index, observation).is_some() {
                    result = Err(Denial::QueryExecutionDenied);
                    break;
                }
                while let Some(observation) = ready.remove(&next_accept) {
                    result = observation.and_then(|(first, second)| {
                        catch_unwind(AssertUnwindSafe(|| accept(next_accept, first, second)))
                            .unwrap_or(Err(Denial::QueryExecutionDenied))
                    });
                    if result.is_err() {
                        break;
                    }
                    next_accept += 1;
                    if next_dispatch < members.len() {
                        if task_sender.send(next_dispatch).is_err() {
                            result = Err(Denial::QueryExecutionDenied);
                            break;
                        }
                        next_dispatch += 1;
                    }
                }
                if result.is_err() {
                    break;
                }
            }
            result
        };
        // Closing both channels before joining prevents blocked workers from
        // escaping the scope, including on denial or partial spawn failure.
        drop(task_sender);
        drop(result_receiver);
        let joined = workers
            .into_iter()
            .fold(true, |joined, worker| worker.join().is_ok() && joined);
        if joined {
            result
        } else {
            Err(Denial::QueryExecutionDenied)
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use worth_relational::facade::identity::PartitionId;

    use super::*;

    #[test]
    fn next_work_starts_before_slowest_eighth_read_finishes() {
        let members = (0usize..16)
            .map(|i| (EntityId::new(PartitionId::main(), i as u64 + 1, 1), i))
            .collect::<Vec<_>>();
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let release = AtomicBool::new(false);
        let (started, receiver) = mpsc::channel();
        let mut ordered = Vec::new();
        std::thread::scope(|scope| {
            let release_flag = &release;
            let releaser = scope.spawn(move || {
                let observed = receiver.recv_timeout(Duration::from_secs(3)).is_ok();
                release_flag.store(true, Ordering::Release);
                observed
            });
            read_bounded_pairs(
                &members,
                &|index| {
                    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    if *index == 7 {
                        while !release.load(Ordering::Acquire) {
                            std::thread::yield_now();
                        }
                    }
                    if *index == 8 {
                        started.send(()).unwrap();
                    }
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok((*index, *index))
                },
                |index, first, second| {
                    assert_eq!(index, first);
                    assert_eq!(first, second);
                    ordered.push(index);
                    Ok(())
                },
            )
            .unwrap();
            assert!(
                releaser.join().unwrap(),
                "ninth read waited for eighth chunk tail"
            );
        });
        assert_eq!(ordered, (0..16).collect::<Vec<_>>());
        assert!(peak.load(Ordering::SeqCst) <= MAX_PAIR_READS_IN_FLIGHT);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn denied_read_joins_workers_and_does_not_accept_later_results() {
        let members = (0usize..16)
            .map(|i| (EntityId::new(PartitionId::main(), i as u64 + 1, 1), i))
            .collect::<Vec<_>>();
        let active = AtomicUsize::new(0);
        let mut accepted = Vec::new();
        let result = read_bounded_pairs(
            &members,
            &|index| {
                active.fetch_add(1, Ordering::SeqCst);
                let answer = if *index == 3 {
                    Err(Denial::QueryExecutionDenied)
                } else {
                    Ok((*index, *index))
                };
                active.fetch_sub(1, Ordering::SeqCst);
                answer
            },
            |index, _, _| {
                accepted.push(index);
                Ok(())
            },
        );
        assert!(matches!(result, Err(Denial::QueryExecutionDenied)));
        assert_eq!(accepted, vec![0, 1, 2]);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn callback_panic_is_typed_denial_and_no_worker_escapes() {
        let members = (0usize..16)
            .map(|i| (EntityId::new(PartitionId::main(), i as u64 + 1, 1), i))
            .collect::<Vec<_>>();
        let active = AtomicUsize::new(0);
        let result = read_bounded_pairs(
            &members,
            &|index| {
                if *index == 3 {
                    panic!("read failure");
                }
                active.fetch_add(1, Ordering::SeqCst);
                active.fetch_sub(1, Ordering::SeqCst);
                Ok((*index, *index))
            },
            |_, _, _| Ok(()),
        );
        assert!(matches!(result, Err(Denial::QueryExecutionDenied)));
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }
}
