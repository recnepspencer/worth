//! Compile-time wire bounds: the most bytes a discriminator encodes to, or
//! `None` when its encoding can grow (it carries text). A structural-identity
//! scheme admits only a discriminator whose bound fits
//! [`MAX_STRUCTURAL_DISCRIMINATOR_BYTES`], so every name minted from it has
//! the same bounded birth at any split depth. Bounds are derived from the
//! field types, never declared, and mirror the encoder byte for byte.

use super::{ApplicationNameDiscriminator, NAME_DISCRIMINATOR_DOMAIN_TAG};

/// Relational's ceiling on a structural-identity discriminator; execution
/// lowering asserts the two agree at compile time.
pub const MAX_STRUCTURAL_DISCRIMINATOR_BYTES: usize = 512;

/// Every length prefix the encoder writes is a big-endian `u64`.
const LEN_PREFIX: usize = 8;

/// The sum of `parts`, or `None` when any part is unbounded.
#[doc(hidden)]
pub const fn bound_sum(parts: &[Option<usize>]) -> Option<usize> {
    let (mut total, mut index) = (0usize, 0);
    while index < parts.len() {
        let Some(part) = parts[index] else {
            return None;
        };
        let Some(next) = total.checked_add(part) else {
            return None;
        };
        total = next;
        index += 1;
    }
    Some(total)
}

/// The largest of `parts` (an enum's variants), or `None` when any is
/// unbounded.
#[doc(hidden)]
pub const fn bound_max(parts: &[Option<usize>]) -> Option<usize> {
    let (mut largest, mut index) = (0usize, 0);
    while index < parts.len() {
        let Some(part) = parts[index] else {
            return None;
        };
        if part > largest {
            largest = part;
        }
        index += 1;
    }
    Some(largest)
}

/// A named field: its tag, its length-prefixed name, then its value.
#[doc(hidden)]
pub const fn field_bound(name: &str, value: Option<usize>) -> Option<usize> {
    bound_sum(&[Some(1 + LEN_PREFIX + name.len()), value])
}

/// An enum variant's tag and length-prefixed name.
#[doc(hidden)]
pub const fn variant_bound(name: &str) -> Option<usize> {
    Some(1 + LEN_PREFIX + name.len())
}

/// The whole canonical bytes of `D`: the domain tag, its length-prefixed
/// identity, then its body. A bounded discriminator, its bound declared as
/// the derive writes it, is admitted by a structural-identity scheme, and
/// its widest value encodes to exactly that bound:
///
/// ```
/// use worth_query_declaration::facade::application_naming::*;
/// struct Step(u64);
/// impl ApplicationNameDiscriminator for Step {
///     const DISCRIMINATOR_IDENTITY: &'static str = "example.step";
///     const MAX_BODY_BYTES: Option<usize> =
///         field_bound("step", <u64 as ApplicationNameDiscriminatorValue>::MAX_ENCODED_BYTES);
///     fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
///         encoder.field("step", &self.0);
///     }
/// }
/// struct Schema;
/// let _ = ApplicationNameSchemeBuilder::<Schema, ()>::for_group("example.step", "Step")
///     .structural_identity::<Step>();
/// assert_eq!(Some(Step(u64::MAX).canonical_bytes().len()), max_encoded_bytes::<Step>());
/// ```
pub const fn max_encoded_bytes<D: ApplicationNameDiscriminator>() -> Option<usize> {
    let header = NAME_DISCRIMINATOR_DOMAIN_TAG.len() + LEN_PREFIX;
    bound_sum(&[
        Some(header + D::DISCRIMINATOR_IDENTITY.len()),
        D::MAX_BODY_BYTES,
    ])
}

/// `D` nested in another discriminator: its tag, its length-prefixed
/// identity and its length-prefixed body.
pub(super) const fn nested_bound<D: ApplicationNameDiscriminator>() -> Option<usize> {
    let framing = 1 + LEN_PREFIX + D::DISCRIMINATOR_IDENTITY.len() + LEN_PREFIX;
    bound_sum(&[Some(framing), D::MAX_BODY_BYTES])
}

/// Whether `D` may tell apart the names of a structural-identity scheme.
pub(crate) const fn discriminator_fits_structural_identity<D: ApplicationNameDiscriminator>() -> bool
{
    matches!(max_encoded_bytes::<D>(), Some(bound) if bound <= MAX_STRUCTURAL_DISCRIMINATOR_BYTES)
}

#[cfg(test)]
#[path = "wire_bound_conformance_trace.rs"]
mod conformance_trace;
