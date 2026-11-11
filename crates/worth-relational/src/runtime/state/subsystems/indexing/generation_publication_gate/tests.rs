//! Coordination kernels, without Native commit or storage authority.
use super::*;

#[test]
fn publication_wait_polls_outside_its_coordination_mutex() {
    let gate = Arc::new(GenerationPublicationGate::default());
    let permit = gate
        .enter(|| {
            let state = gate
                .occupied
                .try_lock()
                .expect("poll does not run under the gate mutex");
            assert!(!*state);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(*gate.occupied.lock().unwrap());
    drop(permit);
    assert!(!*gate.occupied.lock().unwrap());
}

#[test]
fn abandoning_publication_allows_the_next_waiting_owner() {
    let gate = Arc::new(GenerationPublicationGate::default());
    let permit = gate.enter_uncontrolled();
    std::thread::scope(|scope| {
        let (started, waiting) = std::sync::mpsc::channel();
        let gate = &gate;
        let next = scope.spawn(move || {
            let mut first = Some(started);
            let permit = gate
                .enter(|| {
                    if let Some(started) = first.take() {
                        started.send(()).unwrap();
                    }
                    Ok::<_, ()>(())
                })
                .unwrap();
            drop(permit);
        });
        waiting.recv().unwrap();
        drop(permit);
        next.join().unwrap();
    });
    assert!(!*gate.occupied.lock().unwrap());
}
