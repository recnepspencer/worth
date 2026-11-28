//! Schema-affine aftermath authoring and portable installed meaning.

use crate::application_schema::AftermathAssociationAuthority;
use crate::archive_decode_authority::ArchiveDecodeAuthority;

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
}

impl<Schema> Clone for DeclaredApplicationAftermathContract<Schema> {
    fn clone(&self) -> Self {
        Self {
            authority: self.authority,
            mechanism: self.mechanism.clone(),
            reconciliation: self.reconciliation.clone(),
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
            .finish()
    }
}

impl<Schema> PartialEq for DeclaredApplicationAftermathContract<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.authority == other.authority
            && self.mechanism == other.mechanism
            && self.reconciliation == other.reconciliation
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
        )
            .cmp(&(
                &other.authority,
                &other.mechanism,
                &other.reconciliation,
            ))
    }
}

impl<Schema> DeclaredApplicationAftermathContract<Schema> {
    pub const fn runtime_alone(mechanism: DeclaredCorrectionMechanism<Schema>) -> Self {
        Self {
            authority: DeclaredCorrectionAuthority::RuntimeAlone,
            mechanism: Some(mechanism),
            reconciliation: None,
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
        }
    }

    pub const fn not_correctable() -> Self {
        Self {
            authority: DeclaredCorrectionAuthority::NotCorrectable,
            mechanism: None,
            reconciliation: None,
        }
    }

    pub const fn authority(&self) -> DeclaredCorrectionAuthority {
        self.authority
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
        operation: &str,
    ) -> AssociatedAftermathContract {
        let mut fields = Vec::new();
        if let Some(DeclaredCorrectionMechanism::RecordedInverse(inverse)) = &self.mechanism {
            fields.extend(
                inverse
                    .preimage_demand()
                    .loci()
                    .iter()
                    .map(|locus| locus.recipe.clone()),
            );
        }
        let demand_check = validate_expression(operation, self.mechanism.as_ref(), self.reconciliation.as_ref()).and_then(|()| match &self.mechanism {
            Some(DeclaredCorrectionMechanism::RecordedInverse(inverse)) => inverse.preimage_demand().validate().map_err(|cause| crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidCorrectionDemand { correction: operation.to_owned(), cause }),
            _ => Ok(()),
        });
        let contract = demand_check.map(|()| PortableApplicationAftermathContract {
            authority: self.authority,
            mechanism: self.mechanism.map(DeclaredCorrectionMechanism::into_portable),
            reconciliation: self.reconciliation,
            correction_binding: None,
        });
        AssociatedAftermathContract {
            contract,
            fields: fields
                .into_iter()
                .map(|field| (operation.to_owned(), field))
                .collect(),
        }
    }
}

/// Portable, authority-free aftermath meaning stored in a schema declaration.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortableApplicationAftermathContract {
    authority: DeclaredCorrectionAuthority,
    mechanism: Option<PortableCorrectionMechanism>,
    reconciliation: Option<DeclaredReconciliationProcedure>,
    correction_binding: Option<super::DeclaredCorrectionBinding>,
}

impl PortableApplicationAftermathContract {
    /// Reconstructs untrusted descriptive meaning without minting schema
    /// admission or installed-package authority.
    pub fn from_untrusted_fields(
        _: ArchiveDecodeAuthority,
        authority: DeclaredCorrectionAuthority,
        mechanism: Option<PortableCorrectionMechanism>,
        reconciliation: Option<DeclaredReconciliationProcedure>,
    ) -> Self {
        Self {
            authority,
            mechanism,
            reconciliation,
            correction_binding: None,
        }
    }

    pub const fn authority(&self) -> DeclaredCorrectionAuthority {
        self.authority
    }

    pub fn correction_binding(&self) -> Option<&super::DeclaredCorrectionBinding> {
        self.correction_binding.as_ref()
    }

    /// Reconstructs descriptive association; installation still resolves its bindings.
    pub fn with_untrusted_correction_binding(
        mut self,
        binding: super::DeclaredCorrectionBinding,
    ) -> Self {
        self.correction_binding = Some(binding);
        self
    }

    pub const fn mechanism(&self) -> Option<&PortableCorrectionMechanism> {
        self.mechanism.as_ref()
    }

    pub const fn reconciliation(&self) -> Option<&DeclaredReconciliationProcedure> {
        self.reconciliation.as_ref()
    }
}

/// Compiler-local association retains refusals and codec expectations until schema composition.
pub(crate) struct AssociatedAftermathContract {
    pub(crate) contract: Result<
        PortableApplicationAftermathContract,
        crate::application_schema::ApplicationSchemaDeclarationDenial,
    >,
    pub(crate) fields: Vec<(
        String,
        crate::application_schema::ApplicationFieldBindingRecipe,
    )>,
}

fn validate_expression<Schema>(
    correction: &str,
    mechanism: Option<&DeclaredCorrectionMechanism<Schema>>,
    reconciliation: Option<&DeclaredReconciliationProcedure>,
) -> Result<(), crate::application_schema::ApplicationSchemaDeclarationDenial> {
    use super::DeclaredCorrectionMechanismDenial as Cause;
    let check = (|| {
        if let Some(reconciliation) = reconciliation {
            reconciliation.validate()?;
        }
        match mechanism {
            Some(DeclaredCorrectionMechanism::RecordedInverse(inverse)) => {
                if inverse.inverse_operation_slot().trim().is_empty() {
                    return Err(Cause::EmptyInverseOperation);
                }
                inverse.lowering_correspondence().validate()?;
                if !matches!(
                    inverse.postcondition(),
                    super::DeclaredAftermathPostcondition::ExactPriorTruth
                ) {
                    return Err(Cause::RecordedInverseRequiresExactPriorTruth);
                }
                Ok(())
            }
            Some(DeclaredCorrectionMechanism::Compensation(compensation)) => {
                compensation.validate()
            }
            _ => Ok(()),
        }
    })();
    check.map_err(|cause| {
        crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidCorrectionMechanism {
            correction: correction.to_owned(),
            cause,
        }
    })
}
