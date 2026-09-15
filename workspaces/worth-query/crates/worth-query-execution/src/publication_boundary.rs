//! Capability used by the publication audience to drive installed program
//! custody after it has performed source discovery and retained-read checks.

/// Compiler-visible access to program-output progression owned by the
/// publication crate. The host facade intentionally does not re-export this
/// type or its issuer.
pub struct WorthQueryProgramPublicationAccess {
    _private: (),
}

pub fn program_publication_access() -> WorthQueryProgramPublicationAccess {
    WorthQueryProgramPublicationAccess { _private: () }
}
