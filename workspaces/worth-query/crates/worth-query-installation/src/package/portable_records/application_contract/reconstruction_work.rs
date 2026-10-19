//! Complete recursive work observations for retained application contracts.

use worth_foundational::facade::{AspectContract, AspectShape};

use super::*;
mod query_context;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthQueryPortableApplicationContractReconstructionWork {
    pub(crate) logical_bytes: u64,
    pub(crate) nested_entries: u64,
}

impl WorthQueryPortableApplicationContractReconstructionWork {
    fn observe_text(&mut self, value: &str) {
        self.logical_bytes = self
            .logical_bytes
            .saturating_add(crate::package::reconstruction_text_bytes(value));
        self.nested_entries = self.nested_entries.saturating_add(1);
    }

    fn observe_fixed(&mut self, bytes: u64) {
        self.logical_bytes = self.logical_bytes.saturating_add(bytes);
        self.nested_entries = self.nested_entries.saturating_add(1);
    }

    fn observe_contract(&mut self, contract: &AspectContract) {
        self.logical_bytes = self
            .logical_bytes
            .saturating_add(u64::try_from(contract.semantic_byte_width()).unwrap_or(u64::MAX));
        let shape_entries = match contract.shape() {
            AspectShape::Struct(shape) => u64::try_from(shape.fields().len()).unwrap_or(u64::MAX),
            _ => 1,
        };
        self.nested_entries = self
            .nested_entries
            .saturating_add(7)
            .saturating_add(shape_entries);
    }
}

impl WorthQueryPortableNativeAspectContractRecord {
    pub(crate) fn reconstruction_work(
        &self,
    ) -> WorthQueryPortableApplicationContractReconstructionWork {
        let mut work = WorthQueryPortableApplicationContractReconstructionWork::default();
        work.observe_text(&self.schema);
        work.observe_text(&self.entity);
        work.observe_text(self.aspect.as_str());
        work.observe_contract(&self.contract);
        for field in &self.fields {
            work.observe_text(field.as_str());
        }
        work.observe_text(&self.binding.canonical_name());
        work
    }
}

impl WorthQueryPortableApplicationOperationContractRecord {
    pub(crate) fn reconstruction_work(
        &self,
    ) -> WorthQueryPortableApplicationContractReconstructionWork {
        let mut work = WorthQueryPortableApplicationContractReconstructionWork::default();
        work.observe_text(&self.schema);
        work.observe_text(&self.operation);
        work.observe_text(self.input_type.as_str());
        for scope in &self.graph_reads {
            observe_graph_read(&mut work, scope);
        }
        for scope in &self.touches {
            observe_touch(&mut work, scope);
        }
        for emission in &self.emissions {
            work.observe_text(emission);
        }
        if let Some(effect) = &self.external_effect {
            work.observe_text(effect.correlation_family().as_str());
            work.observe_text(effect.effect());
            work.observe_text(effect.payload_type().as_str());
            work.observe_text(effect.protocol().identity().as_str());
            work.observe_fixed(16);
        }
        if let Some(reconciliation) = &self.reconciliation {
            work.observe_text(reconciliation.procedure_slot());
        }
        if let Some(body) = &self.body {
            if let Some(succession) = body.succession() {
                for text in [
                    succession.workflow_spec(),
                    succession.binding(),
                    succession.role(),
                ] {
                    work.observe_text(text);
                }
                work.observe_fixed(1);
                if let Some(matching) = succession.output_match() {
                    for text in [
                        matching.selected_role(),
                        matching.producer_binding(),
                        &matching.producer_role().name,
                        &matching.producer_role().entity,
                    ] {
                        work.observe_text(text);
                    }
                    work.observe_fixed(4);
                }
            }
            for output in body.outputs() {
                work.observe_text(&output.binding);
                work.observe_text(&output.role.name);
                work.observe_text(&output.role.entity);
                work.observe_fixed(6);
                match &output.target {
                    worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Scope => {}
                    worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Creation { key } => { work.observe_text(key); }
                    worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Related { relation, from, to } => {
                        for text in [relation, from, to] { work.observe_text(text); }
                    }
                }
            }
            if let Some(scope) = body.explicit_scope_entity() {
                work.observe_text(scope);
            }
            for requirement in body.requirements() {
                work.observe_text(requirement.identity());
                work.observe_text(requirement.target_name());
                for operand in requirement.operands() {
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
                for expression in requirement
                    .applicability()
                    .into_iter()
                    .chain(std::iter::once(requirement.satisfaction()))
                {
                    work.observe_fixed(u64::try_from(expression.draft().len()).unwrap_or(u64::MAX));
                }
            }
            for step in body.scope_fields() {
                work.observe_text(step.field().entity());
                work.observe_text(step.field().aspect());
                work.observe_text(step.field().field());
                work.observe_text(step.binding().as_str());
                if let Some(value) = step.literal_value() {
                    work.observe_fixed(
                        u64::try_from(value.semantic_byte_width()).unwrap_or(u64::MAX),
                    );
                } else if let Some(member) = step.input_member() {
                    work.observe_text(member);
                    work.observe_fixed(2);
                } else {
                    // Explicit optional Clear retains its family, not an input.
                    work.observe_fixed(2);
                }
            }
            if let Some(creation) = body.creation() {
                if let Some(signing) = creation.signing() {
                    work.observe_text(signing.purpose());
                    for field in [signing.actor(), signing.provenance()] {
                        work.observe_text(field.entity());
                        work.observe_text(field.aspect());
                        work.observe_text(field.field());
                    }
                    work.observe_fixed(1);
                }
                for link in creation.links() {
                    for text in [&link.relation, &link.from, &link.to, &link.key] {
                        work.observe_text(text);
                    }
                    work.observe_fixed(2);
                }
                work.observe_text(creation.entity());
                work.observe_text(creation.key());
                for step in creation.fields() {
                    if let Some(source) = step.related_source() {
                        for text in [&source.relation, &source.from, &source.to] {
                            work.observe_text(text);
                        }
                        work.observe_fixed(1);
                    }
                    work.observe_text(step.field().entity());
                    work.observe_text(step.field().aspect());
                    work.observe_text(step.field().field());
                    work.observe_text(step.binding().as_str());
                    if let Some(value) = step.literal_value() {
                        work.observe_fixed(
                            u64::try_from(value.semantic_byte_width()).unwrap_or(u64::MAX),
                        );
                    } else if let Some(member) = step.input_member() {
                        work.observe_text(member);
                        work.observe_fixed(2);
                    } else if let Some(source) = step.source_field() {
                        work.observe_text(source.entity());
                        work.observe_text(source.aspect());
                        work.observe_text(source.field());
                        work.observe_fixed(2);
                    } else if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreatedField::PrincipalIdentity {
                        mutation_binding, principal_binding, source, ..
                    } = step {
                        for text in [mutation_binding.as_str(), principal_binding.as_str(), source.entity(), source.aspect(), source.field()] {
                            work.observe_text(text);
                        }
                        work.observe_fixed(2);
                    }
                }
            }
        }
        work
    }
}

#[cfg(test)]
#[path = "reconstruction_work/body_tests.rs"]
mod body_tests;

fn observe_graph_read(
    work: &mut WorthQueryPortableApplicationContractReconstructionWork,
    scope: &WorthQueryPortableOperationGraphReadScope,
) {
    work.observe_fixed(1);
    match scope {
        WorthQueryPortableOperationGraphReadScope::Entity { schema, entity } => {
            work.observe_text(schema);
            work.observe_text(entity);
        }
        WorthQueryPortableOperationGraphReadScope::NativeProjection {
            schema,
            entity,
            aspect,
            contract,
            mask,
        } => {
            work.observe_text(schema);
            work.observe_text(entity);
            work.observe_text(aspect.as_str());
            work.observe_contract(contract);
            for path in mask.paths() {
                work.observe_fixed(1);
                for field in path.fields() {
                    work.observe_text(field.as_str());
                }
            }
        }
        WorthQueryPortableOperationGraphReadScope::Relation {
            schema,
            relation,
            from,
            to,
        } => {
            for value in [schema, relation, from, to] {
                work.observe_text(value);
            }
        }
    }
}

fn observe_touch(
    work: &mut WorthQueryPortableApplicationContractReconstructionWork,
    scope: &WorthQueryPortableOperationTouchScope,
) {
    work.observe_fixed(1);
    match scope {
        WorthQueryPortableOperationTouchScope::CreateEntity { schema, entity }
        | WorthQueryPortableOperationTouchScope::DeleteEntity { schema, entity } => {
            work.observe_text(schema);
            work.observe_text(entity);
        }
        WorthQueryPortableOperationTouchScope::WriteField {
            schema,
            entity,
            contract,
            field_path,
        } => {
            work.observe_text(schema);
            work.observe_text(entity);
            work.observe_contract(contract);
            for field in field_path.fields() {
                work.observe_text(field.as_str());
            }
        }
        WorthQueryPortableOperationTouchScope::LinkRelation {
            schema,
            relation,
            from,
            to,
        }
        | WorthQueryPortableOperationTouchScope::UnlinkRelation {
            schema,
            relation,
            from,
            to,
        } => {
            for value in [schema, relation, from, to] {
                work.observe_text(value);
            }
        }
    }
}
