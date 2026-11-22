//! Public operation reads retain the exact issued selection before releasing values.
use super::super::decision_reads::structural_walks::WalkRecordDenial;
use super::*;
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Read one complete Native component after proving every member visible
    /// through an installed path to this operation's exact admitted root.
    /// Work grows with that cohort and its reverse visibility witnesses.
    pub fn component_of<Set: OperationReads<Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>, Denial> {
        let mut answer = self.tracked_components(set, Some(member))?;
        answer
            .pop()
            .ok_or_else(|| Denial::new(Kind::InvalidComponent, set.name()))
    }
    /// Enumerate the complete forward member universe and return whole Native
    /// cohorts. A partially visible cohort refuses the entire census.
    pub fn components<Set: OperationReads<Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<Vec<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>>, Denial> {
        self.tracked_components(set, None)
    }
    /// Count through the same complete census, including empty-set dependencies.
    pub fn component_count<Set: OperationReads<Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<usize, Denial> {
        Ok(self.tracked_components(set, None)?.len())
    }
    fn tracked_components<Set: OperationReads<Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        member: Option<&WorthQueryInvariantEntityIdentity<Schema, Entity>>,
    ) -> Result<Vec<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>>, Denial> {
        let result = (|| {
            let (answer, body) = self.select_component_answer(set, member)?;
            let admission = self
                .admission_identity
                .ok_or_else(|| Denial::new(Kind::UndeclaredDecisionTarget, set.name()))?;
            let contracts = self
                .admitted_contracts
                .clone()
                .ok_or_else(|| Denial::new(Kind::UndeclaredDecisionTarget, set.name()))?;
            let execution = *self.reader.execution;
            let result = execution.run_native_with_checkpoint(|context| {
                self.decision_facts.record_component(
                    &body,
                    admission,
                    contracts,
                    self.reader.authority_identity,
                    context,
                )
            });
            match result {
                Ok((Ok(()), _)) => {}
                Ok((Err(WalkRecordDenial::Execution(cause)), _)) => {
                    return Err(self.reader.walk_stop(cause, set.name()))
                }
                Ok((Err(WalkRecordDenial::Conflict), _)) => {
                    return Err(Denial::new(Kind::ForeignIdentity, set.name()))
                }
                Err(stop) => return Err(self.reader.walk_stop(stop.into(), set.name())),
            }
            for facts in body.facts() {
                for entity in &facts.entities {
                    self.reader.realized_scope.record(*entity);
                }
            }
            Ok(answer)
        })();
        // Failure can precede a key. A handler swallowing it must never seal.
        if result.is_err() {
            self.decision_facts.fail_component();
        }
        result
    }
}
