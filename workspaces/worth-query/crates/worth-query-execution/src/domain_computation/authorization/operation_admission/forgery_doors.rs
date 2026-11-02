//! Test-only doors that forge brands around foreign or altered cores.
//!
//! Production has no core-to-brand conversion; these doors exist so the
//! forgery court can present what erasure would otherwise make unreachable.

use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::equality_type_id::EqualityTypeId;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    /// Keeps this brand's tag and typed inputs around `donor`'s core.
    pub(in crate::domain_computation) fn forge_with_core_of_for_test<
        DonorOperation,
        DonorInput,
        DonorScope,
    >(
        self,
        donor: WorthQueryAdmittedApplicationOperation<
            Schema,
            DonorOperation,
            DonorInput,
            DonorScope,
        >,
    ) -> Self {
        Self {
            core: donor.core,
            ..self
        }
    }

    pub(in crate::domain_computation) fn flip_core_seal_byte_for_test(&mut self) {
        self.core.operation_authority_identity_bytes[0] ^= 0x01;
    }

    pub(in crate::domain_computation) fn forge_tag_input_type_for_test<Forged: 'static>(&mut self) {
        self.tag.input_type = EqualityTypeId::of::<Forged>();
    }

    pub(in crate::domain_computation) fn forge_tag_operation_type_for_test<Forged: 'static>(
        &mut self,
    ) {
        self.tag.operation_type = EqualityTypeId::of::<Forged>();
    }

    /// Offers `successor`'s core as this brand's next phase through the real
    /// rewrap, so continuity is judged where production judges it.
    pub(in crate::domain_computation) fn rewrap_successor_for_test(
        self,
        successor: Self,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        let predecessor = self.core.admission_identity;
        Self::rewrap(
            predecessor,
            successor.core,
            self.tag,
            self.capability_input,
            self.original_input,
        )
        .map_err(|(denial, _)| denial)
    }
}
