//! One exact head retirement, including its dirty-frontier insertion.
use super::HistoryReachabilityIndex;
use crate::identity::CompositeCommitIdentity;

impl HistoryReachabilityIndex {
    pub(in crate::history) fn try_retire_product_head(
        &mut self,
        identity: &CompositeCommitIdentity,
        seated: bool,
    ) -> Result<(), ()> {
        let slot = self.records.get(identity).expect("live head record");
        let mut held = slot.try_lock().map_err(|_| ())?;
        let record = held.as_mut().expect("installed head record");
        let mut counters = self.counters.try_lock().map_err(|_| ())?;
        let direct = record.direct_protections.checked_sub(1).expect("live head");
        let seat = record
            .product_head_protections
            .checked_sub(usize::from(seated))
            .expect("seated head");
        let enqueue = direct == 0 && !record.dirty;
        // A dirty target is already queued. In particular, target==tail never
        // acquires the same mutex twice.
        let mut tail = if enqueue {
            self.dirty_tail
                .as_ref()
                .map(|tail| {
                    assert_ne!(tail, identity);
                    self.records
                        .get(tail)
                        .expect("dirty tail")
                        .try_lock()
                        .map_err(|_| ())
                })
                .transpose()?
        } else {
            None
        };
        // Every required lock is now held. Nothing below can refuse or call
        // an owner, allocate a registry entry, or drop capability custody.
        record.direct_protections = direct;
        record.product_head_protections = seat;
        counters.record_direct_protection_release();
        if enqueue {
            record.dirty = true;
            record.dirty_previous = self.dirty_tail.clone();
            record.dirty_next = None;
            if let Some(tail) = tail.as_mut() {
                tail.as_mut().expect("installed dirty tail").dirty_next = Some(identity.clone());
            } else {
                self.dirty_head = Some(identity.clone());
            }
            self.dirty_tail = Some(identity.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::reference_test_fixture as fixture;
    #[test]
    fn head_close_tail_busy_is_atomic_and_success_disarms_original_drop() {
        let (mut fixture, catalog, root) = fixture::installed_root();
        let successor = fixture::install_ordinary(&mut fixture, &catalog, root.as_ref());
        let index = catalog.state.lock().unwrap().reachability.clone();
        while index.lock().unwrap().pop_changed().is_some() {}
        // Genuine release of another protection queues only the successor.
        drop(catalog.protect_product_head(successor.as_ref()).unwrap());
        let snapshot = fixture::initial_snapshot(&mut fixture, root.clone());
        let history = catalog.try_retain_head_for_close(&snapshot).unwrap();
        let protection = fixture::product_head_protection(&fixture, &catalog, snapshot);
        let cell = crate::branch::ProductBranchReferenceCell::new(protection).unwrap();
        cell.seat_head(true);
        let (target, tail, counters) = {
            let held = index.lock().unwrap();
            (
                held.records.get(root.identity()).unwrap().clone(),
                held.records.get(successor.identity()).unwrap().clone(),
                held.counters.clone(),
            )
        };
        let before = target.lock().unwrap().clone();
        let counts = *counters.lock().unwrap();
        let cancellation = worth_execution::CancellationSource::new();
        let policy =
            crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
        let request = worth_execution::ExecutionRequest::serial(&policy);
        let held = tail.lock().unwrap();
        assert_eq!(
            cell.try_retire_for_close(&request).unwrap_err(),
            crate::lifecycle::RuntimeWorldCloseDenial::Busy
        );
        assert_eq!(*target.lock().unwrap(), before);
        assert_eq!(*counters.lock().unwrap(), counts);
        cancellation.cancel();
        assert!(matches!(
            cell.try_retire_for_close(&request),
            Err(crate::lifecycle::RuntimeWorldCloseDenial::Stopped(
                worth_execution::MapKernelStop::Cancelled
            ))
        ));
        assert_eq!(*target.lock().unwrap(), before);
        drop(held);
        let retry = crate::test_execution_policy::serial_request();
        cell.try_retire_for_close(&worth_execution::ExecutionRequest::serial(&retry))
            .unwrap();
        let retired = target.lock().unwrap().clone().unwrap();
        assert_eq!(retired.direct_protections, 0);
        assert_eq!(retired.product_head_protections, 0);
        assert!(retired.dirty);
        assert_eq!(
            index.lock().unwrap().dirty_tail.as_ref(),
            Some(root.identity())
        );
        assert_eq!(
            tail.lock().unwrap().as_ref().unwrap().dirty_next.as_ref(),
            Some(root.identity())
        );
        let released = counters.lock().unwrap().direct_protection_releases();
        drop(cell);
        assert_eq!(
            counters.lock().unwrap().direct_protection_releases(),
            released
        );
        assert_eq!(released, counts.direct_protection_releases() + 1);
        drop(history);
    }
}
