//! Related-subject selection shares the ordinary adjacency seal and attestation evaluator.
use super::{
    invalid, ApplicationSchema, Direction, HandlerExecutionDenial, Operand,
    PreparedRequirementOperand, QueryRequestExecution,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as AttemptDenial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_foundational::expression_api::ExpressionValue;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in super::super) fn related_signed_evidence_operand<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Operand,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRequirementOperand, HandlerExecutionDenial> {
        let Operand::RelatedSignedEvidence {
            relation,
            from,
            to,
            evidence,
        } = source
        else {
            unreachable!("related attestation selector")
        };
        // Fresh readmission binds endpoints. Retain a defensive native check too.
        if to != &evidence.lifecycle.source_entity {
            return Err(invalid());
        }
        let collection = Operand::RelatedItemCount {
            relation: relation.clone(),
            from: from.clone(),
            to: to.clone(),
            direction: Direction::Outgoing,
        };
        let selected =
            self.observed_requirement_collection(root, &collection, execution, context)?;
        let entity_id = match selected.ids.as_slice() {
            [] => {
                return Ok(PreparedRequirementOperand {
                    value: ExpressionValue::bool(false),
                    _reservation: selected.reservation,
                })
            }
            [entity_id] => *entity_id,
            _ => {
                return Err(HandlerExecutionDenial::new(AttemptDenial::new(
                    Kind::AmbiguousRelation,
                    relation,
                )))
            }
        };
        let kind = self.reader.layout.entity_kind(to).ok_or_else(invalid)?;
        let subject = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind,
            entity: to.clone().into(),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        // The subject's provenance affinity and current evidence are checked by
        // the same evaluator as a direct scoped requirement. Never merge rounds.
        self.signed_evidence_operand(&subject, evidence, execution, context)
    }
}
