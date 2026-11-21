use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

impl WorthQueryApplicationIdempotencyBinding {
    pub fn matches_workflow_instance(
        &self,
        identity: &[u8; 32],
    ) -> bool {
        self.workflow_instance_identity.as_ref() == Some(identity)
    }
    pub const fn bind_workflow_instance(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_instance_identity = Some(*identity);
        self
    }
}

pub fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-instance", identity);
    }
}
