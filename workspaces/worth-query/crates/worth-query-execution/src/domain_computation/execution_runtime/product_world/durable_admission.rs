//! Directory capability admission uses Host's one exhaustive support policy.
use super::WorthQueryProductRuntime;
use worth_foundational::facade::DurableStateFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDurableStateAdmissionDenial {
    PolicyUnavailable,
    UndeclaredSupplierCapabilities,
    Unsupported {
        family: DurableStateFamily,
        supported_from_phase: u8,
    },
}
impl std::fmt::Display for WorthQueryDurableStateAdmissionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "directory capability admission refused: {self:?}")
    }
}
impl std::error::Error for WorthQueryDurableStateAdmissionDenial {}

impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn admit_directory_family(
        &self,
        family: DurableStateFamily,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        if self.owner.owner_identity().durable().is_none() {
            return Ok(());
        }
        self.admit_directory_requirement(family, 0)
    }

    fn admit_directory_requirement(
        &self,
        family: DurableStateFamily,
        required_phase: u8,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        if !self.is_directory_home() {
            return Ok(());
        }
        let support = self
            .directory_support
            .ok_or(WorthQueryDurableStateAdmissionDenial::PolicyUnavailable)?;
        match support(family, required_phase) {
            None => Ok(()),
            Some(supported_from_phase) => Err(WorthQueryDurableStateAdmissionDenial::Unsupported {
                family,
                supported_from_phase,
            }),
        }
    }
    pub(in crate::domain_computation) fn admit_directory_capability(
        &self,
        capability: crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        let (families, phase) = capability.requirements();
        for &family in families {
            self.admit_directory_requirement(family, phase)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation) fn is_directory_home(&self) -> bool {
        self.owner.owner_identity().durable().is_some()
    }
}
