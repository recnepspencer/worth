use super::{
    invoice::hash_invoice,
    stimulus::{grid, install, measured, Counted},
};
use crate::content::original;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Barrier,
    },
    time::Instant,
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::content::*;

struct FinishObservation<'a>(&'a AtomicBool);
impl Drop for FinishObservation<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

#[test]
fn content_frame_scales_have_independent_hashes_linear_cold_work_and_constant_warm_work() {
    // Python hashlib/OpenSSL over literal LE framing and i - n/2 sample values.
    let golden = [
        "b5f7b593224cd8bc3d834a3538a62d79aede53436db98e84a2031aae22764ffd",
        "de88338e576680ec007496f7cc9bb885d5e31c97c4926107eeccf0cf82431d2f",
        "87d9abda52ccc291ce509c6b7e0e9c449944ebfa46819be10e1db277636e4db7",
    ];
    let mut cold = Vec::new();
    let mut warm = Vec::new();
    for (mib, golden) in [1, 2, 4].into_iter().zip(golden) {
        let cancel = CancellationSource::new();
        let original = original(&cancel);
        let execution = ExecutionRequest::serial(&original);
        let codec = Counted::new();
        let format = install(codec.clone());
        let input = grid(mib * 1024 * 1024);
        let peak = AtomicU64::new(0);
        let done = AtomicBool::new(false);
        let ready = Barrier::new(2);
        let (frame, encode_work, encode_time) = std::thread::scope(|scope| {
            let observer = scope.spawn(|| {
                ready.wait();
                while !done.load(Ordering::Acquire) {
                    peak.fetch_max(
                        original.memory().unwrap().charged_memory_bytes(),
                        Ordering::SeqCst,
                    );
                    std::thread::yield_now();
                }
                // The encoder holds its actual frame until this observation.
                // Even a late-scheduled observer cannot miss the retained grant.
                peak.fetch_max(
                    original.memory().unwrap().charged_memory_bytes(),
                    Ordering::SeqCst,
                );
            });
            ready.wait();
            // Unwinding an encoder regression must also release the observer.
            let sampling = FinishObservation(&done);
            let started = Instant::now();
            let (frame, work) = measured(&original, || {
                RelationalContentFrame::encode(&execution, &format, &input)
            });
            let elapsed = started.elapsed();
            drop(sampling);
            // Explicit join is the completion handshake. The result still owns
            // its original grant until the observer has sampled it.
            observer.join().unwrap();
            (frame.unwrap(), work, elapsed)
        });
        assert_eq!(frame.bytes().len(), mib * 1024 * 1024);
        assert_eq!(frame.identity().to_string(), golden);
        assert_eq!(codec.encoded.load(Ordering::SeqCst), frame.payload().len());
        assert_eq!(
            codec.validation_billed.load(Ordering::SeqCst),
            frame.payload().len()
        );
        assert_eq!(
            codec.verified.load(Ordering::SeqCst),
            input.samples.len() * 8
        );
        // Full scan of the original numerical list, independent of canonical parsing.
        let (words, remainder) = frame.payload()[8..].as_chunks::<8>();
        assert!(remainder.is_empty());
        assert_eq!(words.len(), input.samples.len());
        for (word, expected) in words.iter().zip(&input.samples) {
            assert_eq!(i64::from_le_bytes(*word), *expected);
        }
        assert!(peak.load(Ordering::SeqCst) >= frame.bytes().len() as u64);
        let hash = hash_invoice(&original, &frame);
        let started = Instant::now();
        let (read, read_work) = measured(&original, || {
            RelationalContentFrame::readmit(&execution, &format, frame.identity(), frame.bytes())
        });
        let read = read.unwrap();
        let read_time = started.elapsed();
        let (equal, equal_work) = measured(&original, || frame.same_content(&execution, &read));
        assert_eq!(equal, Ok(true));
        assert_eq!(equal_work, 65 + 2 * frame.bytes().len() as u64);
        let (equal, warm_work) = measured(&original, || frame.same_content(&execution, &frame));
        assert_eq!(equal, Ok(true));
        assert_eq!(warm_work, 65);
        for offset in [4095, 4096, frame.bytes().len() - 1] {
            let mut corrupt = frame.bytes().to_vec();
            corrupt[offset] ^= 1;
            let before = original.memory().unwrap().charged_memory_bytes();
            assert_eq!(
                RelationalContentFrame::readmit(&execution, &format, frame.identity(), &corrupt)
                    .unwrap_err(),
                RelationalContentDenial::IntegrityMismatch
            );
            assert_eq!(original.memory().unwrap().charged_memory_bytes(), before);
        }
        let retained = original.memory().unwrap().charged_memory_bytes();
        eprintln!("content {mib}MiB encode={encode_work} read={read_work} exact={equal_work} warm={warm_work} hash={hash} retained_two_frames={retained} observed_encode_peak_original_bytes={} encode_us={} read_us={}", peak.load(Ordering::SeqCst), encode_time.as_micros(), read_time.as_micros());
        cold.push((encode_work, read_work));
        warm.push(warm_work);
        drop((read, frame));
        assert_eq!(original.memory().unwrap().charged_memory_bytes(), 0);
    }
    assert_eq!(warm, [65, 65, 65]);
    for pair in cold.windows(2) {
        assert!(pair[1].0 > pair[0].0 && pair[1].0 < 3 * pair[0].0);
        assert!(pair[1].1 > pair[0].1 && pair[1].1 < 3 * pair[0].1);
    }
}
