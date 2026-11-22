pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::dependency_identity::*;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::dependency_identity::canonical_encoding;

pub fn authentication_observation_identity(
    operation: [u8; 32],
    facts: &(impl crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence
          + ?Sized),
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<
    (
        [u8; 32],
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    super::signed_creation::WorthQuerySignedCreationPreparationDenial,
> {
    let (rule, entries) = canonical_encoding::observation_identity_basis(operation, facts, || {
        super::signed_creation::identity::check_interruption(context)
    })?;
    super::signed_creation::canonical::derive(
        entries,
        canonical_encoding::DOMAIN,
        rule,
        request,
        context,
    )
}
