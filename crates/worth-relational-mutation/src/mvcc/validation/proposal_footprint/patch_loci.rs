//! Authored aspect loci shared by exact-field and semantic-touch projections.
use worth_foundational::facade::{
    FieldKey, PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

pub(super) enum AuthoredPatchLocus<'a> {
    Whole(&'a PortableAspectContractBasis),
    Field {
        basis: &'a PortableAspectContractBasis,
        field: &'a FieldKey,
    },
}

pub(super) fn authored_patch_loci(
    patch: &PortableRecordAspectPatch,
) -> impl Iterator<Item = AuthoredPatchLocus<'_>> {
    patch.operations().iter().flat_map(|operation| {
        let basis = operation.basis();
        let (whole, fields) = match operation {
            PortableAspectPatchOperation::PatchFields {
                selected_fields, ..
            } => (None, selected_fields.as_slice()),
            PortableAspectPatchOperation::SetWhole { .. }
            | PortableAspectPatchOperation::ClearWhole { .. } => {
                (Some(AuthoredPatchLocus::Whole(basis)), &[][..])
            }
        };
        whole.into_iter().chain(
            fields
                .iter()
                .map(move |field| AuthoredPatchLocus::Field { basis, field }),
        )
    })
}
