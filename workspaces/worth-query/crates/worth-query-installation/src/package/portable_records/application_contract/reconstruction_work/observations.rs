//! Retained source and expression metadata for all body observations.
use super::WorthQueryPortableApplicationContractReconstructionWork;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyRequirementOperand,
};
impl WorthQueryPortableApplicationContractReconstructionWork {
    pub(super) fn observe_body_observations(&mut self, body: &ApplicationOperationBodyProgram) {
        let work = self;
        for step in body.path_value_assignments() {
            work.observe_text(&step.target.canonical_record());
            work.observe_fixed(u64::try_from(step.target.path.steps().len()).unwrap_or(u64::MAX));
            work.observe_text(step.binding.as_str());
            work.observe_text(&step.value);
        }
        for requirement in body.observation_requirements() {
            work.observe_text(requirement.identity());
            work.observe_text(requirement.target_name());
            work.observe_operands(requirement.operands());
            for expression in requirement
                .applicability()
                .into_iter()
                .chain(std::iter::once(requirement.satisfaction()))
            {
                work.observe_fixed(u64::try_from(expression.draft().len()).unwrap_or(u64::MAX));
            }
        }

        for value in body.values() {
            work.observe_text(value.identity());
            work.observe_text(value.binding().as_str());
            work.observe_operands(value.operands());
            work.observe_fixed(u64::try_from(value.expression().draft().len()).unwrap_or(u64::MAX));
            work.observe_text(value.expression().semantic_material());
        }
    }
    fn observe_operands(&mut self, operands: &[ApplicationOperationBodyRequirementOperand]) {
        let work = self;
        for operand in operands {
            use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource as Source;
            work.observe_text(operand.name());
            match operand.source() {
                Source::QueryContext {
                    mutation_binding,
                    query,
                } => {
                    work.observe_query_context(mutation_binding, query);
                }
                Source::SignedEvidence(source)
                | Source::RelatedSignedEvidence {
                    evidence: source, ..
                } => {
                    for text in [
                        &source.mutation_binding,
                        &source.operation,
                        &source.evidence_entity,
                        source.provenance.entity(),
                        source.provenance.aspect(),
                        source.provenance.field(),
                        &source.purpose,
                        &source.producer_meaning,
                        &source.lifecycle.history_relation,
                        &source.lifecycle.current_relation,
                        &source.lifecycle.source_entity,
                        &source.lifecycle.consequence_relation,
                        source.lifecycle.consequence_field.entity(),
                        source.lifecycle.consequence_field.aspect(),
                        source.lifecycle.consequence_field.field(),
                    ] {
                        work.observe_text(text);
                    }
                    work.observe_fixed(1);
                    if let Source::RelatedSignedEvidence {
                        relation, from, to, ..
                    } = operand.source()
                    {
                        for text in [relation, from, to] {
                            work.observe_text(text);
                        }
                    }
                }
                Source::PrincipalIdentity {
                    scope,
                    mutation_binding,
                    principal_binding,
                    field,
                    binding,
                    ..
                } => {
                    for text in [
                        scope.as_str(),
                        mutation_binding.as_str(),
                        principal_binding.as_str(),
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        binding.as_str(),
                    ] {
                        work.observe_text(text);
                    }
                    work.observe_fixed(2);
                }
                Source::Field { field, binding, .. } => {
                    work.observe_text(field.entity());
                    work.observe_text(field.aspect());
                    work.observe_text(field.field());
                    work.observe_text(binding.as_str());
                    work.observe_fixed(4);
                }
                Source::RelatedFieldValues {
                    relation,
                    from,
                    to,
                    field,
                    binding,
                    ..
                } => {
                    for text in [
                        relation.as_str(),
                        from.as_str(),
                        to.as_str(),
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        binding.as_str(),
                    ] {
                        work.observe_text(text);
                    }
                    work.observe_fixed(2);
                }
                Source::RelatedItemCount {
                    relation, from, to, ..
                } => {
                    work.observe_text(relation);
                    work.observe_text(from);
                    work.observe_text(to);
                    work.observe_fixed(1);
                }
            }
        }
    }
}
