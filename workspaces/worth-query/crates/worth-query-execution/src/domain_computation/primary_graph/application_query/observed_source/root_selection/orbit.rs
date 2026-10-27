//! The declared slot and original aggregate body of one complete orbit answer.
use super::*;
use crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
use worth_query_declaration::facade::application_query::ApplicationQueryOrbitMeaning;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_query) struct DeclaredOrbitSelection {
    pub(in crate::domain_computation::primary_graph::application_query) meaning:
        Arc<ApplicationQueryOrbitMeaning>,
    pub(in crate::domain_computation::primary_graph::application_query) aggregate:
        Arc<AggregateObservation>,
    pub(in crate::domain_computation::primary_graph::application_query) binding:
        ApplicationSchemaBindingIdentity,
}
impl DeclaredOrbitSelection {
    pub(super) fn append_descriptor(&self, digest: &mut Sha256) {
        digest.update(b"declared-whole-orbit:v1");
        for text in [
            self.meaning.query_type(),
            self.meaning.slot_type(),
            self.meaning.relation(),
            self.meaning.from(),
            self.meaning.to(),
            self.meaning.output_name(),
        ] {
            encode_text(digest, text);
        }
        digest.update(self.aggregate.native().definition().bytes());
        digest.update(self.aggregate.native().identity().bytes());
        digest.update((self.aggregate.native().len() as u64).to_be_bytes());
    }
}
