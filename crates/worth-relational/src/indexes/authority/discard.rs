use crate::indexes::data::{
    DerivedIndexDiscardDenial, DerivedIndexDiscardOutcome, DerivedIndexDiscardRequest,
};

use super::IndexAuthority;

impl IndexAuthority<'_> {
    /// Remove all currently catalogued generations of one installed index.
    ///
    /// Definitions, authoritative roots, history and unique-field enforcement
    /// remain intact. Readers that already retained a generation can finish;
    /// fresh selection must find a later publication or refuse cold access.
    /// An in-flight build may publish after this operation's catalog removal.
    ///
    /// This is not a durable tombstone or a total physical-memory reclamation:
    /// readers, build outcomes and historical envelope artifacts may share
    /// entry backing. A checkpoint captured afterward records the then-current
    /// catalog; subsequent recovery replay may publish valid index artifacts.
    pub fn discard_generations(
        &self,
        request: DerivedIndexDiscardRequest,
    ) -> Result<DerivedIndexDiscardOutcome, DerivedIndexDiscardDenial> {
        self.runtime.indexes.discard_generations(request)
    }
}
