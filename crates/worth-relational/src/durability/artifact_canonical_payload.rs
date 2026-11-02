//! Canonical-form check shared by Relational's framed durable artifacts:
//! re-serialize the admitted record and compare it, byte for byte and without
//! allocating, against the digested payload. Equal bytes make the payload
//! digest a faithful identity of the record, so two encodings of one commit
//! or one branch transition can never carry two digests.

use std::io::{self, Write};

use serde::Serialize;

/// True when `payload` is exactly the canonical encoding of `canonical`.
///
/// Cost: one serializer pass over the record; O(payload bytes) comparison.
pub(crate) fn is_canonical_payload(canonical: &impl Serialize, payload: &[u8]) -> bool {
    let mut comparison = PayloadComparison {
        remaining: payload,
        diverged: false,
        context: None,
        stop: None,
    };
    let compared = rmp_serde::encode::write_named(&mut comparison, canonical);
    compared.is_ok() && !comparison.diverged && comparison.remaining.is_empty()
}

pub(crate) fn is_canonical_payload_with_original_context(
    canonical: &impl Serialize,
    payload: &[u8],
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
    let mut comparison = PayloadComparison {
        remaining: payload,
        diverged: false,
        context: Some(context),
        stop: None,
    };
    let compared = rmp_serde::encode::write_named(&mut comparison, canonical);
    if let Some(stop) = comparison.stop {
        return Err(stop);
    }
    Ok(compared.is_ok() && !comparison.diverged && comparison.remaining.is_empty())
}

struct PayloadComparison<'a, 'context, 'scope, 'lease> {
    remaining: &'a [u8],
    diverged: bool,
    context: Option<&'context mut worth_execution::MapKernelContext<'scope, 'lease>>,
    stop: Option<crate::execution::RelationalExecutionDenialCause>,
}

impl Write for PayloadComparison<'_, '_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.stop.is_some() {
            return Err(io::Error::other("original canonical comparison stopped"));
        }
        for chunk in input.chunks(4096) {
            if let Some(context) = &mut self.context
                && let Err(cause) =
                    crate::execution::request_stage::checkpoint(context, 2 * chunk.len() as u64)
            {
                self.stop = Some(cause.into());
                return Err(io::Error::other("original canonical comparison stopped"));
            }
            match self.remaining.strip_prefix(chunk) {
                Some(rest) if !self.diverged => {
                    self.remaining = rest;
                }
                _ => {
                    self.diverged = true;
                    return Err(io::Error::other("payload diverges from canonical encoding"));
                }
            }
        }
        Ok(input.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
