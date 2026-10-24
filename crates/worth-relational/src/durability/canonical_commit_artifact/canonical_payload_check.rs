//! Canonical-form check for a readmitted payload: re-serialize the admitted
//! commit and compare it, byte for byte and without allocating, against the
//! digested payload. Equal bytes make the payload digest a faithful identity
//! of the commit, so two encodings of one commit can never carry two digests.

use std::io::{self, Write};

use serde::Serialize;

/// True when `payload` is exactly the canonical encoding of `canonical`.
///
/// Cost: one serializer pass over the commit; O(payload bytes) comparison.
pub(super) fn is_canonical_payload(canonical: &impl Serialize, payload: &[u8]) -> bool {
    let mut comparison = PayloadComparison {
        remaining: payload,
        diverged: false,
    };
    let compared = rmp_serde::encode::write_named(&mut comparison, canonical);
    compared.is_ok() && !comparison.diverged && comparison.remaining.is_empty()
}

struct PayloadComparison<'a> {
    remaining: &'a [u8],
    diverged: bool,
}

impl Write for PayloadComparison<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        match self.remaining.strip_prefix(input) {
            Some(rest) if !self.diverged => {
                self.remaining = rest;
                Ok(input.len())
            }
            _ => {
                self.diverged = true;
                Err(io::Error::other("payload diverges from canonical encoding"))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
