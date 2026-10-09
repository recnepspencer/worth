//! A read's ceiling comes from the fact that declares its artifact; no caller
//! can state one as a number:
//! ```compile_fail
//! use worth_store_physical_backend::ArtifactCeiling;
//! let _forged = ArtifactCeiling::from(4096_u64);
//! ```
//! ```compile_fail
//! use worth_store_physical_backend::ArtifactCeiling;
//! use worth_store_physical_format::RecordArtifactFile;
//! let _forged = ArtifactCeiling {
//!     file: RecordArtifactFile::RootManifest { generation: 1 },
//!     extent: todo!(),
//! };
//! ```
//! A grant is minted only by the budget that owns its dimension: no other
//! code can record that budget's grant, and no grant is built field by field:
//! ```compile_fail
//! use worth_foundational::LimitDimension;
//! use worth_proof::Performed;
//! use worth_store_physical_backend::{ReadGrant, UnchargedReadAuthority};
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! struct Borrowed;
//! impl LimitDimension for Borrowed {
//!     type Authority = UnchargedReadAuthority;
//! }
//! let _forged = ReadGrant::granted(
//!     Borrowed,
//!     Performed::record(&UnchargedReadAuthority::witness(), 1 << 20),
//! );
//! ```
//! ```compile_fail
//! use worth_store_physical_backend::{ReadGrant, Uncharged};
//! let _forged: ReadGrant<Uncharged> = ReadGrant { bound: None };
//! ```
