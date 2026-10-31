//! Failed root admission keeps the actual readmitted component custody.
#[derive(Debug)]
pub struct RefusedWorldRootReadmission {
    pub(crate) cause: Box<super::WorldReadmissionDenial>,
    pub(crate) basis: crate::basis::AdmittedCompositeRuntimeWorldBasis,
}

impl RefusedWorldRootReadmission {
    pub fn cause(&self) -> &super::WorldReadmissionDenial {
        &self.cause
    }

    pub fn retains_component_custody(&self) -> bool {
        // Inspect the actual retained basis; this description grants no admission.
        self.basis.owner_identity().durable().is_some()
    }
}
