use super::stimulus::{install, measured, NAME};
use std::{
    any::Any,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{ExecutionRequest, SerialRequest};
use worth_foundational::facade::ContentFormat;
use worth_relational::facade::content::*;

#[derive(Debug)]
struct Reject {
    descriptor: ContentFormat,
    calls: AtomicUsize,
}
impl RelationalContentCodec for Reject {
    fn format(&self) -> &ContentFormat {
        &self.descriptor
    }
    fn payload_length(
        &self,
        _: &dyn Any,
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        Err(RelationalContentDenial::InputTypeMismatch)
    }
    fn encode(
        &self,
        _: &dyn Any,
        _: &mut RelationalContentSink<'_>,
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        Err(RelationalContentDenial::InputTypeMismatch)
    }
    fn validate(
        &self,
        _: &[u8],
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(RelationalContentDenial::NonCanonicalContent)
    }
    fn verify_encoding(
        &self,
        _: &dyn Any,
        _: &[u8],
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        Err(RelationalContentDenial::InputTypeMismatch)
    }
}
// Real funded-record readmission differs only after version validation: v1
// hashes then refuses in the zero-Work validator; v2 refuses before hashing.
// Neither path allocates the descriptor wrapper. This isolates the hash invoice.
pub(super) fn hash_invoice(original: &SerialRequest, frame: &RelationalContentFrame) -> u64 {
    let mut work = Vec::new();
    let before = original.memory().unwrap().charged_memory_bytes();
    for version in [1, 2] {
        let codec = Arc::new(Reject {
            descriptor: ContentFormat::new(NAME, version).unwrap(),
            calls: AtomicUsize::new(0),
        });
        let format = install(codec.clone());
        let (result, charged) = measured(original, || {
            RelationalContentFrame::readmit_record(
                &ExecutionRequest::serial(original),
                &format,
                &frame.record(),
            )
        });
        assert_eq!(
            result.unwrap_err(),
            if version == 1 {
                RelationalContentDenial::NonCanonicalContent
            } else {
                RelationalContentDenial::UnsupportedVersion
            }
        );
        assert_eq!(
            codec.calls.load(Ordering::SeqCst),
            usize::from(version == 1)
        );
        assert_eq!(original.memory().unwrap().charged_memory_bytes(), before);
        work.push(charged);
    }
    let delta = work[0].checked_sub(work[1]).unwrap();
    assert_eq!(
        delta,
        b"worth.native.content.v1\0".len() as u64 + frame.bytes().len() as u64
    );
    delta
}
