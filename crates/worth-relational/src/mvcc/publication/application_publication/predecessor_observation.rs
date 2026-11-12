//! Read classification borrows only the predecessor already bound by Native admission.
use super::*;
use crate::identity::data::EntityId;
use worth_execution::ExecutionRequest;
impl crate::mvcc::ValidatedRelationalProposal {
    /// The exact running writer's admitted predecessor; descriptive selectors issue no custody.
    pub fn output_observation_predecessor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        producer: &str,
        dependency: &[u8; 32],
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        fn own_predecessor<'a>(
            publication: &'a RequiredOutputPublication,
            producer: &str,
            dependency: &[u8; 32],
        ) -> Option<&'a RelationalRequiredOutputPublication> {
            match publication {
                RequiredOutputPublication::Preserve { predecessor, .. }
                    if predecessor.origin.source_scope_owned
                        && publication.lineage_attribution() == Some((producer, *dependency)) =>
                {
                    Some(predecessor)
                }
                _ => None,
            }
        }
        fn select(
            publication: &RequiredOutputPublication,
            producer: &str,
            dependency: &[u8; 32],
            context: &mut worth_execution::MapKernelContext<'_, '_>,
        ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
            crate::execution::request_stage::checkpoint(context, 1)
                .map_err(custody::execution_stop)?;
            if let Some(predecessor) = own_predecessor(publication, producer, dependency) {
                return Ok(Some(predecessor.clone()));
            }
            if let RequiredOutputPublication::CoupledAuthored { group } = publication {
                for part in &group.parts {
                    if let Some(publication) = part.publication.owned()
                        && let Some(prior) = select(publication, producer, dependency, context)?
                    {
                        return Ok(Some(prior));
                    }
                }
            }
            Ok(None)
        }
        let Some(admission) = &self.state.required_output_publication else {
            return Ok(None);
        };
        // The existing single non-owned path enters no new charged scope.
        // One eligibility owner supplies both this constant header decision
        // and each participant's charged lookup in a new coupled group.
        if !matches!(
            admission.publication,
            RequiredOutputPublication::CoupledAuthored { .. }
        ) && own_predecessor(&admission.publication, producer, dependency).is_none()
        {
            return Ok(None);
        }
        execution
            .run_native_with_checkpoint(|context| {
                select(&admission.publication, producer, dependency, context)
            })
            .map_err(custody::execution_stop)?
            .0
    }
}

impl RelationalRequiredOutputPublication {
    /// Canonical member coverage of an already admitted complete predecessor.
    pub fn contains_output_entity(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
        let result = execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, 1)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                let work = u64::from(usize::BITS - self.origin.outputs.len().leading_zeros()) + 1;
                crate::execution::request_stage::checkpoint(context, work)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                Ok(self.origin.outputs.binary_search(&entity).is_ok())
            })
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        result.0
    }
}
