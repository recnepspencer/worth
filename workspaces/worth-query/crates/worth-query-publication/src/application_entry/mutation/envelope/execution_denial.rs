//! Entry failures keep the original typed cause and the exact envelope subject.
use crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryEditEnvelopeSubject {
    Envelope {
        program: String,
    },
    Stage {
        ordinal: usize,
        binding: &'static str,
    },
}
#[derive(Debug)]
pub struct WorthQueryEditEnvelopeExecutionDenial {
    subject: WorthQueryEditEnvelopeSubject,
    cause: WorthQueryPerformedMutationExecutionDenial,
}
impl WorthQueryEditEnvelopeExecutionDenial {
    pub fn subject(&self) -> WorthQueryEditEnvelopeSubject {
        self.subject.clone()
    }
    pub fn cause(&self) -> &WorthQueryPerformedMutationExecutionDenial {
        &self.cause
    }
    pub(super) fn stage(
        ordinal: usize,
        binding: &'static str,
        cause: WorthQueryPerformedMutationExecutionDenial,
    ) -> Self {
        Self {
            subject: WorthQueryEditEnvelopeSubject::Stage { ordinal, binding },
            cause,
        }
    }
    pub(in crate::application_entry) fn envelope(
        program: &str,
        cause: WorthQueryPerformedMutationExecutionDenial,
    ) -> Self {
        Self {
            subject: WorthQueryEditEnvelopeSubject::Envelope {
                program: program.to_owned(),
            },
            cause,
        }
    }
}
impl std::fmt::Display for WorthQueryEditEnvelopeExecutionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.subject, self.cause)
    }
}
impl std::error::Error for WorthQueryEditEnvelopeExecutionDenial {}
