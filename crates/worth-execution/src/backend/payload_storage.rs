//! Payload admission is an explicit strategy of the existing map lifecycle.
use crate::ChargedBytes;

pub(super) enum PayloadStorage<T, R, E> {
    Native,
    Metered {
        input: fn(&T) -> u64,
        result: fn(&R) -> u64,
        error: fn(&E) -> u64,
    },
}

impl<T, R, E> PayloadStorage<T, R, E> {
    pub(super) fn input_measure(&self) -> Option<fn(&T) -> u64> {
        match self {
            Self::Native => None,
            Self::Metered { input, .. } => Some(*input),
        }
    }
    pub(super) fn result_measure(&self) -> Option<fn(&R) -> u64> {
        match self {
            Self::Native => None,
            Self::Metered { result, .. } => Some(*result),
        }
    }
    pub(super) fn error_measure(&self) -> Option<fn(&E) -> u64> {
        match self {
            Self::Native => None,
            Self::Metered { error, .. } => Some(*error),
        }
    }
}

impl<T: ChargedBytes, R: ChargedBytes, E: ChargedBytes> PayloadStorage<T, R, E> {
    pub(super) fn metered() -> Self {
        Self::Metered {
            input: T::additional_charged_bytes,
            result: R::additional_charged_bytes,
            error: E::additional_charged_bytes,
        }
    }
}
