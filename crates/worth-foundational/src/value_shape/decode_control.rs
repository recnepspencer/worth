//! First-cause reporting through the existing borrowed decode admission boundary.
use super::ValueCodecDenial as D;

pub trait ValueDecodeControl {
    fn requires_admission(&self) -> bool {
        true
    }
    fn admit(&mut self, work: usize, bytes: usize) -> Result<(), D>;
    /// Preserve a known codec failure before a Serde adapter erases its type.
    fn reject(&mut self, cause: D) -> D {
        cause
    }
}

impl<F: FnMut(usize, usize) -> Result<(), D>> ValueDecodeControl for F {
    fn admit(&mut self, work: usize, bytes: usize) -> Result<(), D> {
        self(work, bytes)
    }
}

pub type ValueDecodeAdmission<'a> = dyn ValueDecodeControl + 'a;

pub fn value_decode_error<E: serde::de::Error>(
    cause: D,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> E {
    E::custom(match admission {
        Some(admission) => admission.reject(cause),
        None => cause,
    })
}

pub(super) struct RecordingAdmission<'a, 'b> {
    pub(super) inner: &'a mut ValueDecodeAdmission<'b>,
    pub(super) refusal: Option<D>,
    pub(super) require_admission: bool,
}
impl ValueDecodeControl for RecordingAdmission<'_, '_> {
    fn requires_admission(&self) -> bool {
        self.require_admission
    }
    fn admit(&mut self, work: usize, bytes: usize) -> Result<(), D> {
        if let Some(cause) = self.refusal {
            return Err(cause);
        }
        self.inner
            .admit(work, bytes)
            .map_err(|cause| self.reject(cause))
    }
    fn reject(&mut self, cause: D) -> D {
        *self.refusal.get_or_insert(cause)
    }
}
