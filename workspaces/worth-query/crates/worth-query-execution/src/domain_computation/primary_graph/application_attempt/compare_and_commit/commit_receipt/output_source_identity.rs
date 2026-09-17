use super::WorthQueryApplicationCommitReceipt;

impl WorthQueryApplicationCommitReceipt {
    /// The immutable source of a required output can be described by a fresh
    /// receipt or by an authorized idempotency replay of that same publication.
    pub(in crate::domain_computation::primary_graph) fn same_retained_output_source_as(
        &self,
        other: &Self,
    ) -> bool {
        let publication = self.committed_product_publication();
        let other_publication = other.committed_product_publication();
        self.runtime_authority() == other.runtime_authority()
            && self.provider_runtime_instance_id() == other.provider_runtime_instance_id()
            && self.outcome_identity() == other.outcome_identity()
            && self.commit_reference() == other.commit_reference()
            && self.basis_descriptor() == other.basis_descriptor()
            && self.installed_operation() == other.installed_operation()
            && self.principal_scope() == other.principal_scope()
            && self.idempotency_binding() == other.idempotency_binding()
            && self.output_correspondence() == other.output_correspondence()
            && publication.product_branch() == other_publication.product_branch()
            && publication.product_incarnation() == other_publication.product_incarnation()
            && publication.product_generation() == other_publication.product_generation()
            && publication.composite_commit() == other_publication.composite_commit()
            && publication.publication_attempt() == other_publication.publication_attempt()
            && publication.relational_commit() == other_publication.relational_commit()
            && publication.relational_posture() == other_publication.relational_posture()
            && publication.signal_posture() == other_publication.signal_posture()
            && publication.signal_publication() == other_publication.signal_publication()
            && publication.conditional_definition_generation()
                == other_publication.conditional_definition_generation()
    }
}
