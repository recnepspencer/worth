//! Leased original invoices remain separate from serial-accounted telemetry.
use super::stimulus::{grid, install, Counted};
use crate::leased_preparation::ordinary_context::{authority, lease_request_with_work, serial};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling};
use worth_relational::facade::{content::*, transactions::RelationalExecutionDenialCause};

#[test]
fn content_leased_frame_scales_keep_original_work_and_custody() {
    let _serial = serial();
    let mut cold = Vec::new();
    for mib in [1, 2, 4] {
        let cancellation = CancellationSource::new();
        let lease = authority()
            .request_lease(lease_request_with_work(
                32 * 1024 * 1024,
                cancellation.token(),
                100_000_000,
            ))
            .unwrap();
        let execution = ExecutionRequest::leased(&lease);
        let format = install(Counted::new());
        let input = grid(mib * 1024 * 1024);
        let (frame, encode) = ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(Some(&lease), None, |_| {
                RelationalContentFrame::encode(&execution, &format, &input)
            })
            .unwrap();
        let frame = frame.unwrap();
        let (read, read_work) = ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(Some(&lease), None, |_| {
                RelationalContentFrame::readmit(
                    &execution,
                    &format,
                    frame.identity(),
                    frame.bytes(),
                )
            })
            .unwrap();
        let read = read.unwrap();
        let (same, warm) = ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(Some(&lease), None, |_| {
                frame.same_content(&execution, &frame)
            })
            .unwrap();
        assert_eq!(same, Ok(true));
        assert_eq!(warm.charged_work(), 65);
        assert!(lease.charged_memory_bytes() >= (2 * frame.bytes().len()) as u64);
        let (words, remainder) = read.payload()[8..].as_chunks::<8>();
        assert!(remainder.is_empty());
        assert_eq!(words.len(), input.samples.len());
        for (word, expected) in words.iter().zip(&input.samples) {
            assert_eq!(i64::from_le_bytes(*word), *expected);
        }
        eprintln!(
            "content leased {mib}MiB encode={} read={} warm={} retained_original_bytes={}",
            encode.charged_work(),
            read_work.charged_work(),
            warm.charged_work(),
            lease.charged_memory_bytes()
        );
        cold.push((encode.charged_work(), read_work.charged_work()));
        cancellation.cancel();
        assert_eq!(
            RelationalContentFrame::readmit(&execution, &format, frame.identity(), frame.bytes())
                .unwrap_err(),
            RelationalContentDenial::Execution(RelationalExecutionDenialCause::Cancelled)
        );
        drop((read, frame));
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    for pair in cold.windows(2) {
        assert!(pair[1].0 > pair[0].0 && pair[1].0 < pair[0].0 * 3);
        assert!(pair[1].1 > pair[0].1 && pair[1].1 < pair[0].1 * 3);
    }
}
