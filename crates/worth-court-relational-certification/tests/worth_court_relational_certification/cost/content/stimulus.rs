use crate::content::fixture::{Codec, Grid};
use std::any::Any;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{ExecutionWorkCeiling, SerialRequest};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};
use worth_relational::facade::content::*;

pub(super) const NAME: &str = "exact-grid-costs";
pub(super) fn grid(bytes: usize) -> Grid {
    let count = (bytes - 48) / 8;
    assert_eq!(48 + count * 8, bytes);
    Grid {
        rows: 1,
        columns: count.try_into().unwrap(),
        samples: (0..count).map(|i| i as i64 - (count / 2) as i64).collect(),
    }
}
pub(super) fn measured<T>(original: &SerialRequest, f: impl FnOnce() -> T) -> (T, u64) {
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(original), |_| f())
        .unwrap();
    (result, report.charged_work())
}
pub(super) fn install(codec: Arc<dyn RelationalContentCodec>) -> RelationalContentFormat {
    let mut formats = RelationalContentFormatRegistry::default();
    let id = CanonicalDigestId::new([41; 32]);
    formats.register(id, codec).unwrap();
    formats.get(id).unwrap().clone()
}
#[derive(Debug)]
pub(super) struct Counted {
    codec: Codec,
    pub encoded: AtomicUsize,
    pub validation_billed: AtomicUsize,
    pub verified: AtomicUsize,
}
impl Counted {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            codec: Codec {
                descriptor: ContentFormat::new(NAME, 1).unwrap(),
                alter: false,
            },
            encoded: AtomicUsize::new(0),
            validation_billed: AtomicUsize::new(0),
            verified: AtomicUsize::new(0),
        })
    }
}
impl RelationalContentCodec for Counted {
    fn format(&self) -> &ContentFormat {
        self.codec.format()
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        c: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        self.codec.payload_length(input, c)
    }
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        c: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.encode(input, sink, c)?;
        self.encoded
            .fetch_add(self.codec.payload_length(input, c)?, Ordering::SeqCst);
        Ok(())
    }
    fn validate(
        &self,
        payload: &[u8],
        c: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.validate(payload, c)?;
        // The codec charges the payload, but only inspects dimensions. This is
        // billed work, not a claim to have visited every sample during validation.
        self.validation_billed
            .fetch_add(payload.len(), Ordering::SeqCst);
        Ok(())
    }
    fn verify_encoding(
        &self,
        input: &dyn Any,
        payload: &[u8],
        c: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.verify_encoding(input, payload, c)?;
        self.verified.fetch_add(payload.len() - 8, Ordering::SeqCst);
        Ok(())
    }
}
