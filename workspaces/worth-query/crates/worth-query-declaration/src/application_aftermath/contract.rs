//! Schema-affine aftermath authoring and portable installed meaning.

use crate::application_schema::AftermathAssociationAuthority;

use super::{
    DeclaredCorrectionAuthority, DeclaredCorrectionMechanism, DeclaredReconciliationProcedure,
    PortableCorrectionMechanism,
};

/// Schema-affine aftermath contract for one mutation operation.
///
/// `Schema` remains part of the type until the matching schema's operation
/// builder consumes this contract. This prevents a same-text field locus from
/// another schema from entering the operation association lane.
pub struct DeclaredApplicationAftermathContract<Schema> {
    authority: DeclaredCorrectionAuthority,
    mechanism: Option<DeclaredCorrectionMechanism<Schema>>,
    reconciliation: Option<DeclaredReconciliationProcedure>,
    inverse_binding: Option<super::DeclaredInverseBinding>,
}

impl<Schema> Clone for DeclaredApplicationAftermathContract<Schema> {
    fn clone(&self) -> Self {
        Self {
            authority: self.authority,
            mechanism: self.mechanism.clone(),
            reconciliation: self.reconciliation.clone(),
            inverse_binding: self.inverse_binding.clone(),
        }
    }
}

impl<Schema> std::fmt::Debug for DeclaredApplicationAftermathContract<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DeclaredApplicationAftermathContract")
            .field("authority", &self.authority)
            .field("mechanism", &self.mechanism)
            .field("reconciliation", &self.reconciliation)
            .field("inverse_binding", &self.inverse_binding)
            .finish()
    }
}

impl<Schema> PartialEq for DeclaredApplicationAftermathContract<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.authority == other.authority
            && self.mechanism == other.mechanism
            && self.reconciliation == other.reconciliation
            && self.inverse_binding == other.inverse_binding
    }
}

impl<Schema> Eq for DeclaredApplicationAftermathContract<Schema> {}

impl<Schema> PartialOrd for DeclaredApplicationAftermathContract<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredApplicationAftermathContract<Schema> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.authority,
            &self.mechanism,
            &self.reconciliation,
            &self.inverse_binding,
        )
            .cmp(&(
                &other.authority,
                &other.mechanism,
                &other.reconciliation,
                &other.inverse_binding,
            ))
    }
}

impl<Schema> DeclaredApplicationAftermathContract<Schema> {
    pub const fn runtime_alone(mechanism: DeclaredCorrectionMechanism<Schema>) -> Self {
        Self {
            authority: DeclaredCorrectionAuthority::RuntimeAlone,
            mechanism: Some(mechanism),
            reconciliation: None,
            inverse_binding: None,
        }
    }

    pub const fn runtime_with_external_owner(
        mechanism: DeclaredCorrectionMechanism<Schema>,
        reconciliation: DeclaredReconciliationProcedure,
    ) -> Self {
        Self {
            authority: DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            mechanism: Some(mechanism),
            reconciliation: Some(reconciliation),
            inverse_binding: None,
        }
    }

    pub const fn not_correctable() -> Self {
        Self {
            authority: DeclaredCorrectionAuthority::NotCorrectable,
            mechanism: None,
            reconciliation: None,
            inverse_binding: None,
        }
    }

    pub const fn authority(&self) -> DeclaredCorrectionAuthority {
        self.authority
    }

    /// Derives capture and inverse association from the same typed declaration.
    pub fn recorded_inverse_binding<Correction>() -> Self
    where
        Schema: crate::application_schema::ApplicationSchema,
        Correction: super::ApplicationCorrectionBinding<Schema>,
    {
        Self {
            authority: DeclaredCorrectionAuthority::RuntimeAlone,
            mechanism: Some(super::DeclaredCorrectionMechanism::RecordedInverse(
                super::DeclaredRecordedInverse::from_binding::<Correction>(),
            )),
            reconciliation: None,
            inverse_binding: Some(super::DeclaredInverseBinding::of::<Schema, Correction>()),
        }
    }

    pub const fn mechanism(&self) -> Option<&DeclaredCorrectionMechanism<Schema>> {
        self.mechanism.as_ref()
    }

    pub const fn reconciliation(&self) -> Option<&DeclaredReconciliationProcedure> {
        self.reconciliation.as_ref()
    }

    pub(crate) fn associate_with_operation(
        self,
        _authority: AftermathAssociationAuthority<Schema>,
    ) -> PortableApplicationAftermathContract {
        PortableApplicationAftermathContract {
            authority: self.authority,
            mechanism: self
                .mechanism
                .map(DeclaredCorrectionMechanism::into_portable),
            reconciliation: self.reconciliation,
            inverse_binding: self.inverse_binding,
        }
    }
}

/// Portable, authority-free aftermath meaning stored in a schema declaration.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortableApplicationAftermathContract {
    authority: DeclaredCorrectionAuthority,
    mechanism: Option<PortableCorrectionMechanism>,
    reconciliation: Option<DeclaredReconciliationProcedure>,
    inverse_binding: Option<super::DeclaredInverseBinding>,
}

impl PortableApplicationAftermathContract {
    /// Reconstructs untrusted descriptive meaning without minting schema
    /// admission or installed-package authority.
    pub fn from_untrusted_fields(
        authority: DeclaredCorrectionAuthority,
        mechanism: Option<PortableCorrectionMechanism>,
        reconciliation: Option<DeclaredReconciliationProcedure>,
    ) -> Self {
        Self {
            authority,
            mechanism,
            reconciliation,
            inverse_binding: None,
        }
    }

    pub const fn authority(&self) -> DeclaredCorrectionAuthority {
        self.authority
    }

    pub fn inverse_binding(&self) -> Option<&super::DeclaredInverseBinding> {
        self.inverse_binding.as_ref()
    }

    /// Reconstructs descriptive association; installation still resolves its bindings.
    pub fn with_untrusted_inverse_binding(
        mut self,
        binding: super::DeclaredInverseBinding,
    ) -> Self {
        self.inverse_binding = Some(binding);
        self
    }

    pub const fn mechanism(&self) -> Option<&PortableCorrectionMechanism> {
        self.mechanism.as_ref()
    }

    pub const fn reconciliation(&self) -> Option<&DeclaredReconciliationProcedure> {
        self.reconciliation.as_ref()
    }
}
