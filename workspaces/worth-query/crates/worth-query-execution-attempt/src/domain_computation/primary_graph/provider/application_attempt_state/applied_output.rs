//! Select only actually appended registered writers on the exact admitted applied prefix.
use crate::domain_computation::primary_graph::invariant_projection::{
    consumed_output::{AppliedConsumedOutput, ConsumedOutputEvidence},
    WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind as Kind,
};
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
/// Borrow the actual assembly roster; descriptors never admit a provider attempt.
pub trait AppliedOutputParticipants {
    fn visit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        family: &str,
        source: worth_relational::facade::identity::EntityId,
        consume: &mut dyn FnMut(
            crate::domain_computation::WorthQueryProviderSessionView<'_>,
        ) -> Result<(), WorthQueryCurrentOutputDenial>,
    ) -> Result<(), WorthQueryCurrentOutputDenial>;
    /// Retain a managed Source read a decision of this assembly took from
    /// its committed basis at candidate image `observed`: the assembly marks
    /// every later append that changes it, which is the read's whole
    /// candidate support.
    fn retain_source_alignment(
        &self,
        alignment: std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::AppliedSourceAlignment,
        >,
        observed: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    );
}
pub struct AppliedFamilyOutputs {
    pub outputs: Vec<ConsumedOutputEvidence>,
    pub replaced: Vec<(Option<crate::equality_type_id::EqualityTypeId>, [u8; 32])>,
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub fn applied_outputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        schema: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        family: &str,
        source: worth_relational::facade::identity::EntityId,
        participants: &dyn AppliedOutputParticipants,
        selected: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<AppliedFamilyOutputs, WorthQueryCurrentOutputDenial> {
        let unavailable = || WorthQueryCurrentOutputDenial::new(Kind::OutputUnavailable, family);
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let lineage = self
            .graph
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut found = AppliedFamilyOutputs {
            outputs: Vec::new(),
            replaced: Vec::new(),
        };
        participants.visit(execution, family, source, &mut |session| {
            let state = attempts.attempt_state(session).ok_or_else(unavailable)?;
            let attempt = state.attempt();
            let Some(preparation) = attempt.coupled_preparation() else {
                return Ok(());
            };
            let Some(outputs) = &preparation.outputs else {
                return Ok(());
            };
            let Some(pin) = &preparation.after_own else {
                return Ok(());
            };
            if attempt.affinity().installed_binding() != schema
                || outputs.meaning.family != family
                || outputs.meaning.source_scope != source
                || !selected
                    .candidate_identity()
                    .includes_assembly_prefix(pin.candidate_identity())
            {
                return Ok(());
            }
            let Some((installed_family, role)) = lineage
                .checkpoint_family_role(attempt.output_binding_type())
                .map_err(|_| unavailable())?
            else {
                return Ok(());
            };
            if installed_family != family
                || preparation.completion.source().root() != source
                || preparation.completion.source().checkpoint_identity()
                    != *outputs.meaning.source_epoch.bytes()
                || attempt.idempotency().source_partition_identity()
                    != Some(*outputs.meaning.source_partition.bytes())
            {
                return Err(unavailable());
            }
            found.replaced.push((attempt.output_binding_type(), *outputs.meaning.source_partition.bytes()));
            let Some((_, entity)) = attempt.native_publication_roles().zip(outputs.entities.iter())
                .find(|((actual, posture, _, _), _)| *actual == role && *posture != crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture::Retire) else { return Ok(()); };
            let link = AppliedConsumedOutput::capture(
                execution,
                attempt,
                &outputs.producer,
                &outputs.meaning,
                role,
                *entity,
                selected,
            )
            .map_err(|cause| match cause {
                worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
                    WorthQueryCurrentOutputDenial::new(Kind::ExecutionRequest(cause.into()), family)
                }
                worth_execution::ExecutionRequestArrayDenial::Allocation(_) => {
                    WorthQueryCurrentOutputDenial::new(Kind::WorkBudgetExceeded, family)
                }
            })?;
            found.outputs.push(ConsumedOutputEvidence::Appended(link));
            Ok(())
        })?;
        Ok(found)
    }
}
