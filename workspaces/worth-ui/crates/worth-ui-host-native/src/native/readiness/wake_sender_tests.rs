//! The offscreen wake queue counts every posted wake once, waits for one only
//! until its deadline, and refuses wakes once closed, as a platform loop's
//! proxy does after the loop exits.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{UiNativeOffscreenWakes, UiNativeWakeSender};

#[test]
fn every_posted_wake_is_taken_once() {
    let wakes = Arc::new(UiNativeOffscreenWakes::default());
    let sender = UiNativeWakeSender::Offscreen(Arc::clone(&wakes));
    assert_eq!(sender.send(), Ok(()));
    assert_eq!(sender.clone().send(), Ok(()));
    assert_eq!(wakes.take(None), 2);
    assert_eq!(wakes.take(None), 0);
}

#[test]
fn a_passed_deadline_takes_only_what_is_pending() {
    let wakes = UiNativeOffscreenWakes::default();
    assert_eq!(wakes.take(Some(Instant::now())), 0);
}

#[test]
fn a_wake_from_another_thread_ends_the_wait() {
    let wakes = Arc::new(UiNativeOffscreenWakes::default());
    let sender = UiNativeWakeSender::Offscreen(Arc::clone(&wakes));
    let worker = std::thread::spawn(move || sender.send());
    // The deadline only bounds a broken wake; the posted wake ends the wait
    // long before it.
    let deadline = Instant::now() + Duration::from_secs(60);
    let taken = wakes.take(Some(deadline));
    assert!(Instant::now() < deadline, "the wake ended the wait");
    assert_eq!(worker.join().unwrap(), Ok(()));
    assert_eq!(taken + wakes.take(None), 1);
}

#[test]
fn awaiting_a_wake_leaves_it_pending() {
    let wakes = Arc::new(UiNativeOffscreenWakes::default());
    let sender = UiNativeWakeSender::Offscreen(Arc::clone(&wakes));
    assert!(!wakes.await_posted(Instant::now()));
    let worker = std::thread::spawn(move || sender.send());
    let deadline = Instant::now() + Duration::from_secs(60);
    assert!(wakes.await_posted(deadline));
    assert!(Instant::now() < deadline, "the wake ended the wait");
    assert_eq!(worker.join().unwrap(), Ok(()));
    assert_eq!(wakes.take(None), 1);
}

#[test]
fn a_closed_queue_refuses_wakes() {
    let wakes = Arc::new(UiNativeOffscreenWakes::default());
    let sender = UiNativeWakeSender::Offscreen(Arc::clone(&wakes));
    wakes.close();
    assert_eq!(sender.send(), Err(()));
    assert_eq!(wakes.take(None), 0);
}
