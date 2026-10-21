//! Guard observations over a genuine installed source and prepared input.
use super::*;
impl<Query: 'static, Value: 'static> PreparedProducerInput<Query, Value> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn assert_transport_guards(
        self,
        foreign: InstalledProducerEdition,
    ) {
        assert_ne!(
            self.edition, foreign,
            "real distinct installed producer editions"
        );
        assert_ne!(
            std::any::TypeId::of::<Query>(),
            std::any::TypeId::of::<()>()
        );
        let mut proof = Some(self.disclosure);
        assert!(ValidatedProducerInput {
            edition: self.edition,
            proof: &mut proof
        }
        .take::<Query, Value>(foreign)
        .is_none());
        assert!(
            proof.is_some(),
            "edition refusal does not consume authentic custody"
        );
        assert!(ValidatedProducerInput {
            edition: self.edition,
            proof: &mut proof
        }
        .take::<(), Value>(self.edition)
        .is_none());
        assert!(
            proof.is_some(),
            "foreign Query type does not consume custody"
        );
        assert!(ValidatedProducerInput {
            edition: self.edition,
            proof: &mut proof
        }
        .take::<Query, Value>(self.edition)
        .is_some());
        assert!(proof.is_none(), "the actual typed input is consumed once");
        assert!(ValidatedProducerInput {
            edition: self.edition,
            proof: &mut proof
        }
        .take::<Query, Value>(self.edition)
        .is_none());
    }
}
