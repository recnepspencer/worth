use std::sync::{Arc, Mutex};

/// Shared only across the fallible spawn boundary. Cloning the packet never
/// clones its owner; exactly one path takes the contained value.
pub(super) struct UnacceptedCloseOwner<Owner> {
    owner: Arc<Mutex<Option<Owner>>>,
}

impl<Owner> UnacceptedCloseOwner<Owner> {
    pub(super) fn retain(owner: Owner) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(owner))),
        }
    }

    pub(super) fn worker_packet(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
        }
    }

    pub(super) fn take(&self) -> Owner {
        self.owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("one spawn outcome takes the actual owner")
    }
}

#[cfg(test)]
mod tests {
    use super::UnacceptedCloseOwner;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct DropSentinel(Arc<AtomicUsize>);
    impl Drop for DropSentinel {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn refused_spawn_closure_destruction_preserves_the_unstarted_owner() {
        let drops = Arc::new(AtomicUsize::new(0));
        let packet = UnacceptedCloseOwner::retain(DropSentinel(Arc::clone(&drops)));
        let captured = packet.worker_packet();
        let closure = move || captured.take();
        // Models closure refusal, not an actual OS thread-creation failure.
        drop(closure);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let owner = packet.take();
        drop(packet);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(owner);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn accepted_owner_survives_abandonment_of_the_starting_packet() {
        let drops = Arc::new(AtomicUsize::new(0));
        let packet = UnacceptedCloseOwner::retain(DropSentinel(Arc::clone(&drops)));
        let captured = packet.worker_packet();
        let owner = captured.take();
        drop((packet, captured));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(owner);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
