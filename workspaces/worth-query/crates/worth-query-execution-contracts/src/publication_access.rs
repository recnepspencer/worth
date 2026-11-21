//! Compiler-visible access trait for the execution <-> publication seam.

/// Marker trait implemented by `worth-query-execution`'s
/// `WorthQueryProgramPublicationAccess` so lower crates can accept
/// `&dyn WorthQueryProgramPublicationAccessPort` via unsize coercion without
/// breaking inherent methods on `WorthQueryProgramPublicationAccess`.
pub trait WorthQueryProgramPublicationAccessPort {}
