use crate::archive_decode_authority::ArchiveDecodeAuthority;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::ApplicationCapabilityOperationBinding;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityOperationBindingParts {
    pub operation: String,
    pub operation_identity: WorthQueryPortableTypeIdentity,
    pub input_identity: WorthQueryPortableTypeIdentity,
}

impl ApplicationCapabilityOperationBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityOperationBindingParts,
    ) -> Self {
        Self {
            operation: parts.operation,
            operation_identity: parts.operation_identity,
            input_identity: parts.input_identity,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityOperationBindingParts {
        WorthQueryPortableApplicationCapabilityOperationBindingParts {
            operation: self.operation.clone(),
            operation_identity: self.operation_identity.clone(),
            input_identity: self.input_identity.clone(),
        }
    }
}
