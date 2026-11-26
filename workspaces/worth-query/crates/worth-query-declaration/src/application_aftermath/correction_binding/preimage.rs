//! Read-only recorded material grants no current mutation authority.
use super::super::DeclaredPreImageLocus;

/// Original historical field material. No variant grants current authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationCorrectionFieldPreImage<'a> {
    /// This field was unchanged, outside the capture, or unavailable.
    Uncaptured,
    /// An admitted original read found this individual field absent.
    AbsentField,
    /// An admitted original read found this field's optional parent absent.
    AbsentAspect,
    Value(&'a worth_foundational::facade::AspectValue),
}
/// Read-only original values; this view grants no current read or write authority.
pub trait ApplicationCorrectionPreImage<Schema> {
    /// Historical state on the exact original record bound to this output role.
    /// Only an SDK-sealed absence can return an absence variant. Legacy views
    /// cannot turn a missing value into proof of absence.
    fn field_state_for_output(
        &self,
        role: &crate::application_operation::ApplicationMutationOutputRoleDescriptor,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> ApplicationCorrectionFieldPreImage<'_> {
        self.field_for_output(role, locus).map_or(
            ApplicationCorrectionFieldPreImage::Uncaptured,
            ApplicationCorrectionFieldPreImage::Value,
        )
    }
    /// Enumerate the original members of one installed output family. A suffix
    /// identifies historical material only and grants no current record handle.
    fn family_member_suffix(
        &self,
        _family: &crate::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        _index: usize,
    ) -> Option<&str> {
        None
    }
    fn family_member_entity(
        &self,
        _family: &crate::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        _suffix: &str,
    ) -> Option<worth_foundational::facade::EntityId> {
        None
    }
    fn field_for_family_member(
        &self,
        _family: &crate::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        _suffix: &str,
        _locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        None
    }
    fn output_entity(
        &self,
        _role: &crate::application_operation::ApplicationMutationOutputRoleDescriptor,
    ) -> Option<worth_foundational::facade::EntityId> {
        None
    }

    /// Complete original lifecycle material for a declared relation kind.
    /// None means the locus was not retained; Some(0) is authenticated empty material.
    /// Under live-owner member families the rows are the mutation's own changes
    /// of that kind, so Some(0) there means it changed none.
    fn relation_count(
        &self,
        _locus: &super::super::DeclaredPreImageRelationLocus<Schema>,
    ) -> Option<usize> {
        None
    }
    fn relation_at(
        &self,
        _locus: &super::super::DeclaredPreImageRelationLocus<Schema>,
        _index: usize,
    ) -> Option<super::super::ApplicationCorrectionRelationMaterial> {
        None
    }

    fn field(
        &self,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue>;

    /// Historical state of one declared locus: its value, or an authenticated
    /// absence of an optional field (or its optional aspect) the original
    /// read observed. Only an SDK-sealed absence can return an absence variant.
    fn field_state(
        &self,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> ApplicationCorrectionFieldPreImage<'_> {
        self.field(locus).map_or(
            ApplicationCorrectionFieldPreImage::Uncaptured,
            ApplicationCorrectionFieldPreImage::Value,
        )
    }

    /// Read one declared locus on the exact original record bound to this output
    /// role. An absent optional role yields None; equal loci on co-anchors never
    /// substitute for one another. This grants historical values only.
    fn field_for_output(
        &self,
        _role: &crate::application_operation::ApplicationMutationOutputRoleDescriptor,
        _locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        None
    }
}

/// Reads and decodes one recorded pre-image field using the column's declared
/// scalar codec, preserving typed missing and invalid pre-image denials.
pub fn decode_preimage_field<Schema, Field>(
    preimage: &dyn ApplicationCorrectionPreImage<Schema>,
    field: Field,
) -> Result<Field::Value, super::ApplicationCorrectionDerivationDenial>
where
    Field: crate::application_schema::fragment::RecordSelection,
{
    let raw = preimage
        .field(&DeclaredPreImageLocus::from_field(field))
        .ok_or(super::ApplicationCorrectionDerivationDenial::MissingPreImage)?;
    <Field::Binding as crate::application_schema::ApplicationReadableScalarValueBinding>::decode(
        raw,
    )
    .map_err(|_| super::ApplicationCorrectionDerivationDenial::InvalidPreImage)
}

/// Reads and decodes one recorded optional pre-image field: `Some(value)` when
/// the original record held a value, `None` when the original read
/// authenticated the field (or its optional aspect) as absent.
///
/// An uncaptured locus refuses `MissingPreImage`, never `None`: absence must
/// be proven by the recorded original read, not inferred.
pub fn decode_optional_preimage_field<Schema, Field>(
    preimage: &dyn ApplicationCorrectionPreImage<Schema>,
    field: Field,
) -> Result<Option<Field::Value>, super::ApplicationCorrectionDerivationDenial>
where
    Field: crate::application_schema::fragment::RecordSelection,
{
    match preimage.field_state(&DeclaredPreImageLocus::from_field(field)) {
        ApplicationCorrectionFieldPreImage::Value(raw) => {
            <Field::Binding as crate::application_schema::ApplicationReadableScalarValueBinding>::decode(raw)
                .map(Some)
                .map_err(|_| super::ApplicationCorrectionDerivationDenial::InvalidPreImage)
        }
        ApplicationCorrectionFieldPreImage::AbsentField
        | ApplicationCorrectionFieldPreImage::AbsentAspect => Ok(None),
        ApplicationCorrectionFieldPreImage::Uncaptured => {
            Err(super::ApplicationCorrectionDerivationDenial::MissingPreImage)
        }
    }
}
