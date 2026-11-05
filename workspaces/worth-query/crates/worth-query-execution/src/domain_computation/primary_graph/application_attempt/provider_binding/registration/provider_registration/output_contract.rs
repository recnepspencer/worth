//! Output meaning of the exact registered provider effect program.

use crate::equality_type_id::EqualityTypeId as TypeId;

use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture;

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn seal_output_correspondence_from_publication(
        &self,
        publication: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.effects
            .seal_output_correspondence_from_publication(publication)
    }
    pub(in crate::domain_computation::primary_graph) fn native_publication_roles(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture, &str, &worth_relational::facade::transactions::EntityReference)> + Clone{
        self.effects.native_publication_roles()
    }

    pub(in crate::domain_computation::primary_graph) fn native_witness_roles(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, WorthQueryApplicationOutputPosture, &str)> + Clone
    {
        self.effects.native_witness_roles()
    }

    pub(in crate::domain_computation::primary_graph) const fn output_binding_type(
        &self,
    ) -> Option<TypeId> {
        self.effects.output_binding_type()
    }
}

impl super::WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn admit_output_correspondence(
        &mut self,
        prepared: &worth_relational::facade::mvcc::RelationalPreparedOutputPublications,
    ) -> Result<
        (),
        (
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputProjectionDenial,
            String,
        ),
    > {
        self.effects.admit_output_correspondence(prepared)
    }
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn admit_owned_output_correspondence(
        &mut self,
        prepared: &worth_relational::facade::mvcc::RelationalPreparedOutputPublications,
        producer: &str,
        meaning: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Result<
        (),
        (
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputProjectionDenial,
            String,
        ),
    > {
        self.effects
            .admit_owned_output_correspondence(prepared, producer, meaning)
    }
}
