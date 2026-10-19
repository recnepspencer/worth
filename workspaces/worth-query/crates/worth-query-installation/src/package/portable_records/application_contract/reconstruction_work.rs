//! Complete recursive work observations for retained application contracts.

use worth_foundational::facade::{AspectContract, AspectShape};

use super::*;
mod external_emission;
mod observations;
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
            if let Some(emission) = body.external_emission() {
                work.observe_external_emission(emission);
            }
            work.observe_body_observations(body);
            for step in body.scope_fields() {
                work.observe_text(step.field().entity());
                work.observe_text(step.field().aspect());
                work.observe_text(step.field().field());
                work.observe_text(step.binding().as_str());
                if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyScopeField::AssignValue { value, .. } = step {
                    work.observe_text(value);
                } else if let Some(value) = step.literal_value() {
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
                if creation.subject_impact_bound().is_some() {
                    work.observe_fixed(8);
                }
                for step in creation.fields() {
                    if let Some(source) = step.path_source() {
                        work.observe_text(&source.canonical_record());
                        work.observe_fixed(
                            u64::try_from(source.path.steps().len()).unwrap_or(u64::MAX),
                        );
                    }
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
                    if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreatedField::Computed { value, .. } = step {
                        work.observe_text(value);
                    } else if let Some(value) = step.literal_value() {
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
