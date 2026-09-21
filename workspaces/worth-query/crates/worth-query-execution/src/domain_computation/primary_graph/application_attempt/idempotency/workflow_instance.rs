use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_instance(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_instance_identity = Some(*identity);
        self
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-instance", identity);
    }
}
