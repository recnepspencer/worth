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
