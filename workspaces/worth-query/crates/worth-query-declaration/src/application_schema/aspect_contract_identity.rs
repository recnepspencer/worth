//! Stable semantic identity carried by an application aspect declaration.

use worth_foundational::facade::{AspectContractRevision, AspectIdentity};

/// Schema-declared aspect marker identity used to mint exact typed references
/// and installed native contracts.
pub trait ApplicationAspectMarkerIdentity<Schema, Entity> {
    const IDENTIFIER: &'static str;
    const ASPECT_IDENTITY: AspectIdentity;
    const CONTRACT_REVISION: AspectContractRevision;
}
