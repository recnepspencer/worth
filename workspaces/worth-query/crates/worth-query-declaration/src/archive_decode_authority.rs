//! Internal seam for Query's untrusted raw-part decoders.
//!
//! Every `from_untrusted*` constructor in Query's authority packages demands an
//! [`ArchiveDecodeAuthority`]. Neither this module nor the witness is
//! re-exported through [`crate::facade`], `worth-query-decl` or
//! `worth-query-host`, and boundary-check keeps consumer crates from depending
//! on this package, so application code can neither name the witness nor call
//! a raw-part decoder. Applications author meaning through typed declarations.
//!
//! The witness decides only who may call a decoder. It adds no trust to what a
//! decoder reads: every decoded value is still validated by its owner.

/// Zero-sized witness that the caller is Query's own archive, installation,
/// execution, testkit or court code. It cannot be built outside this module.
#[derive(Clone, Copy, Debug)]
pub struct ArchiveDecodeAuthority {
    _sealed: (),
}

/// The witness Query's internal archive and installation code passes to
/// `from_untrusted*` decoders.
pub const ARCHIVE_DECODE: ArchiveDecodeAuthority = ArchiveDecodeAuthority { _sealed: () };
