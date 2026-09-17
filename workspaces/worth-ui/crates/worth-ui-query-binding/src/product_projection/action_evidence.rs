#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScalarProjectionActionEvidence {
    pub(crate) source_revision: u64,
    pub(crate) status: String,
    pub(crate) query_receipt_digest: String,
    pub(crate) affected_live_view_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiScalarProjectionActionPreconditionDenial {
    SourceRevisionMismatch,
}

impl WorthUiScalarProjectionActionEvidence {
    pub(crate) fn from_application_publication(
        source_revision: u64,
        status: String,
        publication: &crate::WorthUiStatusPublication,
    ) -> Self {
        use sha2::{Digest, Sha256};

        let receipt = publication.query_receipt().inspect();
        let basis = receipt.basis();
        let mut digest = Sha256::new();
        digest.update(b"worth.ui.status-action-publication.v1");
        digest.update(receipt.query_identity().as_bytes());
        digest.update(receipt.parameter_binding_identity().as_bytes());
        digest.update(basis.runtime_instance().to_le_bytes());
        digest.update(basis.branch().as_bytes());
        digest.update(basis.version().to_le_bytes());
        digest.update(status.as_bytes());
        Self {
            source_revision,
            status,
            query_receipt_digest: format!("{:x}", digest.finalize()),
            affected_live_view_ids: vec![receipt.query_identity().to_owned()],
        }
    }

    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn query_receipt_digest(&self) -> &str {
        &self.query_receipt_digest
    }

    pub fn affected_live_view_ids(&self) -> &[String] {
        &self.affected_live_view_ids
    }
}
