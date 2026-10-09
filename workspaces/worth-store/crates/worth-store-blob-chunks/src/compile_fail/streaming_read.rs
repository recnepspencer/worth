//! Copied digest strings cannot satisfy bounded content verification:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobStreamingVerifiedRead;
//! fn requires_verified_read(_: BlobStreamingVerifiedRead) {}
//! let digest = "sha256:copied";
//! requires_verified_read(digest);
//! ```
//! Copied streaming read counters cannot satisfy bounded content verification:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobStreamingReadCounterSnapshot, BlobStreamingVerifiedRead};
//! fn requires_verified_read(_: BlobStreamingVerifiedRead) {}
//! let counters: BlobStreamingReadCounterSnapshot = todo!();
//! requires_verified_read(counters);
//! ```
//! Whole-object expected buffers cannot satisfy bounded content verification:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobStreamingVerifiedRead;
//! fn requires_verified_read(_: BlobStreamingVerifiedRead) {}
//! let expected: Vec<u8> = vec![1, 2, 3];
//! requires_verified_read(expected);
//! ```
