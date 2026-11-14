//! The field truth selected from one declared output; installation binds its role.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PreImageOutputFieldCapture {
    ChangedPreserved,
    CompleteRetired,
}
impl PreImageOutputFieldCapture {
    pub const fn posture(self) -> crate::application_operation::ApplicationMutationOutputPosture {
        use crate::application_operation::ApplicationMutationOutputPosture as Posture;
        match self {
            Self::ChangedPreserved => Posture::Preserve,
            Self::CompleteRetired => Posture::Retire,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortablePreImageFieldOutput {
    role: String,
    entity: String,
    capture: PreImageOutputFieldCapture,
}
impl PortablePreImageFieldOutput {
    /// Untrusted data; association checks the actual role, lifecycle and reads.
    pub fn from_untrusted_fields(
        role: String,
        entity: String,
        capture: PreImageOutputFieldCapture,
    ) -> Self {
        Self {
            role,
            entity,
            capture,
        }
    }
    pub fn role(&self) -> &str {
        &self.role
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub const fn capture(&self) -> PreImageOutputFieldCapture {
        self.capture
    }
}
