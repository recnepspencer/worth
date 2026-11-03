//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
    #[doc(hidden)]
    pub fn bind_source_partition_for_test(&mut self, identity: [u8; 32]) {
        Self::bind_source_partition(self, identity)
    }
}
