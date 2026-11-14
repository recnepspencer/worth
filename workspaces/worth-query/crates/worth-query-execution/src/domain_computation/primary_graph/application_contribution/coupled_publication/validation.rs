//! Every real member is admitted once against the whole candidate.
use super::*;
impl<S: ApplicationSchema>
    crate::domain_computation::primary_graph::provider::CoupledProducerAssembly
    for CoupledApplicationAssembly<'_, '_, '_, S>
{
    fn is_authored_edit(&self) -> bool {
        self.demands.authored_stages.is_some()
    }
    fn authored_stage_sources(&self) -> Vec<worth_relational::facade::identity::EntityId> {
        self.stage_captures()
            .iter()
            .map(|stage| stage.source)
            .collect()
    }
    fn prepared_members(
        &self,
    ) -> Vec<crate::domain_computation::primary_graph::provider::CoupledApplicationMember> {
        let mut members = self
            .authored
            .borrow()
            .iter()
            .map(|writer| {
                let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage.as_ref() else {
                    unreachable!("validated authored stage")
                };
                let mut member = prepared.authored_member(
                    writer.physical,
                    writer.capture.identity,
                    writer.capture.source,
                    writer.capture.binding,
                );
                member.position = writer.position;
                member
            })
            .collect::<Vec<_>>();
        members.extend(self.writers.borrow().iter().map(|writer| {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage.as_ref() else {
                unreachable!("validated group prepares every actual participant")
            };
            let mut member = prepared.member(
                &writer.producer.declaration.identity,
                writer.meaning.clone(),
            );
            member.position = writer.position;
            member
        }));
        members.sort_by_key(|member| member.position);
        members
    }
    fn invariant_requirements(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ) -> Result<
        Vec<worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement>,
        crate::domain_computation::WorthQueryInvariantExecutionFailure,
    > {
        let authored = self.authored.borrow();
        let writers = self.writers.borrow();
        let mut actual = candidate.coupled_participants();
        let mut requirements = Vec::new();
        for writer in authored.iter() {
            if candidate
                .authored_stage_contribution_records(writer.physical, writer.capture.source)
                .is_none()
            {
                return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::missing_coupled_settlement(writer.capture.source));
            }
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.as_ref() else {
                unreachable!("Native applied every authored stage")
            };
            requirements
                .extend(provider.coupled_invariant_requirements(execution, inspected.session())?);
        }
        for writer in writers.iter() {
            let matches = actual.next().is_some_and(|(producer, meaning)| {
                producer == writer.producer.declaration.identity && meaning == &writer.meaning
            });
            if !matches {
                return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::missing_coupled_settlement(writer.meaning.source_scope));
            }
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.as_ref() else {
                unreachable!("Native completed each writer before group judgment")
            };
            requirements
                .extend(provider.coupled_invariant_requirements(execution, inspected.session())?);
        }
        if let Some((_, meaning)) = actual.next() {
            return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::missing_coupled_settlement(meaning.source_scope));
        }
        Ok(requirements)
    }
    fn admit_validated(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        receipts: &crate::domain_computation::primary_graph::provider::ClosedCoupledInvariantReceipts<'_>,
    ) -> Result<(), crate::domain_computation::WorthQueryInvariantExecutionFailure> {
        self.admit_authored(execution, provider, receipts)?;
        for writer in self.writers.borrow_mut().iter_mut() {
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.as_ref() else {
                unreachable!("each actual assembly is approved once")
            };
            provider.admit_coupled_candidate(
                execution,
                inspected.session(),
                receipts,
                &writer.meaning,
            )?;
            let Some(CoupledWriterStage::Inspected(inspected)) = writer.stage.take() else {
                unreachable!("retained actual inspection")
            };
            let prepared = inspected
                .approve(execution, self.phase, self.application)
                .and_then(|approved| {
                    writer.producer.coupled_executor().prepare_coupled_commit(
                        execution,
                        self.phase,
                        self.application,
                        approved,
                    )
                });
            match prepared {
                Ok(approved) => writer.stage = Some(CoupledWriterStage::Prepared(approved)),
                Err(outcome) => {
                    let denial =
                        execution::coupled_preparation_denial(&writer.meaning.family, outcome)
                            .into_output_denial();
                    let detail = denial.subject().to_owned();
                    self.denial.borrow_mut().get_or_insert(denial);
                    return Err(crate::domain_computation::WorthQueryInvariantExecutionFailure::new(
                        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected, detail,
                    ));
                }
            }
        }
        Ok(())
    }
}
