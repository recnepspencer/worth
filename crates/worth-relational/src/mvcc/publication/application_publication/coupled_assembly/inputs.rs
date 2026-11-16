//! Actual prerequisite publication custody, distinct from owned output members.
use super::*;
use crate::publication::data::AuthoredRetirementRefusal as Cause;

/// Request-local carriage of an existing Native publication or an admitted part.
/// Descriptive routes alone cannot construct this input or enroll a writer.
#[derive(Clone, Debug)]
pub struct RelationalRequiredOutputInput {
    proposal: RelationalMutationProposalIdentity,
    issuance: Arc<InputIssuance>,
    origin: InputOrigin,
}

#[derive(Debug)]
pub(super) struct InputIssuance {
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Default)]
pub(crate) struct RequiredOutputInputs {
    pub(crate) meanings: Vec<RequiredOutputPublicationMeaning>,
    pub(crate) memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Clone, Debug)]
enum InputOrigin {
    Current(RelationalRequiredOutputPublication),
    Applied(usize),
    Restored {
        index: usize,
        source: crate::identity::data::EntityId,
    },
}

impl RelationalAuthoredCoupledCandidate<'_> {
    // The inverse prefix has already passed Native's exact payload/endpoint
    // validation. Only its resolved full-generation source can borrow a route.
    pub(super) fn restored_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        original: crate::identity::data::EntityId,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<RelationalRequiredOutputInput, TransactionCommitError> {
        let restoration = self
            .binding
            .owners
            .source
            .restoration()
            .expect("restored owner");
        let index = execution
            .run_native_with_checkpoint(|context| {
                for (index, publication) in restoration.publications.iter().enumerate() {
                    let meaning = &publication.origin.meaning;
                    checkpoint(context, (family.len() + meaning.family.len() + 65) as u64)
                        .map_err(custody::execution_stop)?;
                    if meaning.family == family
                        && meaning.source_scope == original
                        && meaning.source_partition.bytes() == partition
                    {
                        return Ok(index);
                    }
                }
                Err(part::reject(Cause::ForeignCoupledOwner, source))
            })
            .map_err(custody::execution_stop)?
            .0?;
        Ok(RelationalRequiredOutputInput {
            proposal: self.identity.clone(),
            issuance: self.input_issuance(execution)?,
            origin: InputOrigin::Restored { index, source },
        })
    }

    fn input_route<'a>(
        &'a self,
        input: &'a InputOrigin,
    ) -> (
        &'a RequiredOutputPublicationMeaning,
        crate::identity::data::EntityId,
    ) {
        match input {
            InputOrigin::Current(publication) => (
                &publication.origin.meaning,
                publication.origin.meaning.source_scope,
            ),
            InputOrigin::Applied(index) => {
                let meaning = self.parts[*index]
                    .publication
                    .producing_meaning()
                    .expect("admitted earlier part");
                (meaning, meaning.source_scope)
            }
            InputOrigin::Restored { index, source } => (
                &self
                    .binding
                    .owners
                    .source
                    .restoration()
                    .expect("issued restored input")
                    .publications[*index]
                    .origin
                    .meaning,
                *source,
            ),
        }
    }
    fn input_issuance(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<InputIssuance>, TransactionCommitError> {
        if let Some(issuance) = self.input_issuance.borrow().as_ref() {
            return Ok(Arc::clone(issuance));
        }
        let bytes =
            (2 * std::mem::size_of::<usize>() + std::mem::size_of::<InputIssuance>()) as u64;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(custody::execution_stop)?;
        let issuance = Arc::new(InputIssuance { _memory: memory });
        *self.input_issuance.borrow_mut() = Some(Arc::clone(&issuance));
        Ok(issuance)
    }

    /// Carry a publication already authenticated at this candidate's actual C.
    pub fn current_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        publication: &RelationalRequiredOutputPublication,
    ) -> Result<RelationalRequiredOutputInput, TransactionCommitError> {
        let origin = &publication.origin;
        let subject = origin.meaning.source_scope;
        let current = execution
            .run_native_with_checkpoint(|context| {
                checkpoint(context, origin.members.len() as u64 + 1)
                    .map_err(custody::execution_stop)?;
                if origin
                    .members
                    .iter()
                    .all(|member| member.posture == RequiredOutputRolePosture::Retire)
                {
                    return Ok(false);
                }
                if publication.readmitted_at.as_ref() == Some(&self.binding.owners.before) {
                    return Ok::<_, TransactionCommitError>(true);
                }
                for selected in &self.binding.owners.current {
                    checkpoint(
                        context,
                        (origin.meaning.family.len() + selected.origin.meaning.family.len() + 65)
                            as u64,
                    )
                    .map_err(custody::execution_stop)?;
                    if selected.origin.same_publication(origin) {
                        return Ok(true);
                    }
                }
                Ok(false)
            })
            .map_err(custody::execution_stop)?
            .0?;
        if !current
            || publication.runtime_instance_id != self.binding.installed.runtime_instance_id
            || !self
                .binding
                .installed
                .authority
                .owns_required_output_publication(publication.authority())
        {
            return Err(part::reject(Cause::ForeignCoupledOwner, subject));
        }
        Ok(RelationalRequiredOutputInput {
            proposal: self.identity.clone(),
            issuance: self.input_issuance(execution)?,
            origin: InputOrigin::Current(publication.clone()),
        })
    }

    /// Only an actual earlier appended owned publication can supply this input.
    /// Ordinary participants carry no Native-owned publication to retain.
    pub fn applied_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputInput>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                for (index, part) in self.parts.iter().enumerate() {
                    checkpoint(context, meaning.family.len() as u64 + 65)
                        .map_err(custody::execution_stop)?;
                    if part.publication.producing_meaning() == Some(meaning) {
                        return if part.publication.owned().is_some() {
                            Ok(Some(RelationalRequiredOutputInput {
                                proposal: self.identity.clone(),
                                issuance: self.input_issuance(execution)?,
                                origin: InputOrigin::Applied(index),
                            }))
                        } else {
                            Ok(None)
                        };
                    }
                }
                Err(part::reject(
                    Cause::ForeignCoupledOwner,
                    meaning.source_scope,
                ))
            })
            .map_err(custody::execution_stop)?
            .0
    }

    /// Bind the complete closed prerequisite set to its actual performed consumer.
    /// A current read never rewrites the existing publication's recorded inputs.
    pub fn retain_output_inputs(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        consumer: &RelationalRequiredOutputInput,
        parents: &[&RelationalRequiredOutputInput],
    ) -> Result<(), TransactionCommitError> {
        let issuance = self.input_issuance.borrow().as_ref().cloned();
        if consumer.proposal != *self.identity
            || !issuance
                .as_ref()
                .is_some_and(|issuance| Arc::ptr_eq(issuance, &consumer.issuance))
        {
            return Err(super::super::origin_denial(
                "foreign consumed-output proposal",
            ));
        }
        let index = match &consumer.origin {
            InputOrigin::Applied(index) => *index,
            InputOrigin::Current(_) | InputOrigin::Restored { .. } => return Ok(()),
        };
        let part = self
            .parts
            .get(index)
            .ok_or_else(|| super::super::origin_denial("consumed-output part is unavailable"))?;
        let own = part.publication.producing_meaning().expect("admitted part");
        let subject = own.source_scope;
        let bytes = execution
            .run_native_with_checkpoint(|context| {
                let mut bytes = parents
                    .len()
                    .checked_mul(std::mem::size_of::<RequiredOutputPublicationMeaning>())
                    .ok_or_else(|| super::super::origin_denial("consumed-output size overflow"))?;
                for parent in parents {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if parent.proposal != *self.identity
                        || !Arc::ptr_eq(&parent.issuance, &consumer.issuance)
                    {
                        return Err(part::reject(Cause::ForeignCoupledOwner, subject));
                    }
                    if matches!(&parent.origin, InputOrigin::Applied(previous) if *previous >= index) {
                        return Err(part::reject(Cause::ForeignCoupledOwner, subject));
                    }
                    let (meaning, source) = self.input_route(&parent.origin);
                    checkpoint(
                        context,
                        (own.family.len() + meaning.family.len() + 65) as u64,
                    )
                    .map_err(custody::execution_stop)?;
                    if own.family == meaning.family && own.source_scope == source
                        && own.source_partition == meaning.source_partition {
                        return Err(part::reject(Cause::ForeignCoupledOwner, subject));
                    }
                    bytes = bytes.checked_add(meaning.family.len()).ok_or_else(|| {
                        super::super::origin_denial("consumed-output size overflow")
                    })?;
                }
                Ok::<_, TransactionCommitError>(bytes)
            })
            .map_err(custody::execution_stop)?
            .0?;
        let memory = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(custody::execution_stop)?;
        let retained = execution
            .run_native_with_checkpoint(|context| {
                let mut retained: Vec<RequiredOutputPublicationMeaning> = Vec::new();
                retained.try_reserve_exact(parents.len()).map_err(|_| {
                    super::super::origin_denial("consumed-output allocation unavailable")
                })?;
                for parent in parents {
                    let (meaning, source) = self.input_route(&parent.origin);
                    checkpoint(context, (meaning.family.len() + retained.len() + 65) as u64)
                        .map_err(custody::execution_stop)?;
                    let mut found = None;
                    for prior in &retained {
                        checkpoint(
                            context,
                            (prior.family.len() + meaning.family.len() + 65) as u64,
                        )
                        .map_err(custody::execution_stop)?;
                        if prior.family == meaning.family
                            && prior.source_scope == source
                            && prior.source_partition == meaning.source_partition
                        {
                            found = Some(prior);
                            break;
                        }
                    }
                    if let Some(prior) = found {
                        if prior.implementation != meaning.implementation
                            || prior.source_epoch != meaning.source_epoch
                        {
                            return Err(part::reject(Cause::ForeignCoupledOwner, subject));
                        }
                    } else {
                        let mut meaning = meaning.clone();
                        meaning.source_scope = source;
                        retained.push(meaning);
                    }
                }
                let comparisons = retained.len().saturating_mul(
                    usize::BITS as usize - retained.len().leading_zeros() as usize + 1,
                );
                let width = retained
                    .iter()
                    .map(|input| input.family.len())
                    .max()
                    .unwrap_or(0)
                    .saturating_add(65);
                checkpoint(context, comparisons.saturating_mul(width) as u64)
                    .map_err(custody::execution_stop)?;
                retained.sort_by(|a, b| {
                    (&a.family, a.source_scope, a.source_partition).cmp(&(
                        &b.family,
                        b.source_scope,
                        b.source_partition,
                    ))
                });
                Ok(retained)
            })
            .map_err(custody::execution_stop)?
            .0?;
        let part = &mut self.parts[index];
        if let Some(prior) = &part.inputs {
            if prior.meanings != retained {
                return Err(part::reject(Cause::ForeignCoupledOwner, subject));
            }
        } else {
            part.inputs = Some(RequiredOutputInputs {
                meanings: retained,
                memory,
            });
        }
        Ok(())
    }
}
