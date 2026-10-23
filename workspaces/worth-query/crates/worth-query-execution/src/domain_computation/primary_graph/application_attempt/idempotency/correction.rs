//! Immutable correction intent extends the actual inverse input's identity.
use super::WorthQueryApplicationIdempotencyBinding;

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) fn bind_correction_command(
        mut self,
        key: &[u8; 32],
        identity: &[u8; 32],
    ) -> Self {
        self.key_identity = *key;
        self.correction_identity = Some(*identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) fn correction_identity(
        &self,
    ) -> Option<&[u8; 32]> {
        self.correction_identity.as_ref()
    }
}
