//! The existing body grammar interprets every field locus as entity-owned.
use super::super::*;
use crate::application_schema::ApplicationFieldBindingLocus;

fn entity(field: &ApplicationFieldBindingLocus) -> bool {
    field.owner_kind().is_entity()
}

impl ApplicationOperationBodyProgram {
    /// Refuses owner-kind erasure before legacy body interpretation or encoding.
    /// Closed variants and containers are inspected here, including inert nested
    /// descriptions that have not yet earned schema admission.
    pub fn has_only_entity_field_loci(&self) -> bool {
        let Self {
            explicit_scope: _,
            scope_fields,
            creation,
            requirements: _,
            outputs: _,
            succession: _,
            external_emission: _,
            values: _,
            retirements,
            path_input_expectations,
            path_value_assignments,
            reference_inputs,
        } = self;
        scope_fields.iter().all(|step| entity(step.field()))
            && creation
                .as_ref()
                .is_none_or(|creation| creation_fields(creation, 0))
            && self
                .observation_operands()
                .all(|operand| source_fields(operand.source()))
            && retirements.iter().all(|rule| entity(&rule.trigger))
            && path_input_expectations
                .iter()
                .all(|guard| entity(&guard.source.field))
            && path_value_assignments
                .iter()
                .all(|step| entity(&step.target.field))
            && reference_inputs
                .iter()
                .all(|reference| entity(&reference.field))
    }
}

fn creation_fields(creation: &ApplicationOperationBodyCreation, depth: usize) -> bool {
    let ApplicationOperationBodyCreation {
        entity: _,
        key: _,
        fields,
        links,
        signing,
        signing_lifecycle,
        subject_impact_bound: _,
        enrollment: _,
        reuse_current_if_unchanged: _,
        peers,
        path_links: _,
        children,
    } = creation;
    // The admitted grammar has root/peer sites with flat children. Reject
    // deeper inert nesting before it can hide a field in a legacy encoding.
    depth <= 2
        && fields.iter().all(created_field)
        && links.iter().all(|link| match &link.source {
            ApplicationOperationBodyCreationLinkSource::Scope => true,
            ApplicationOperationBodyCreationLinkSource::Principal {
                mutation_binding: _,
                principal_binding: _,
                identity,
            } => entity(identity),
        })
        && signing
            .as_ref()
            .is_none_or(|spec| spec.has_only_entity_field_loci())
        && signing_lifecycle.as_ref().is_none_or(lifecycle_fields)
        && peers.iter().all(|peer| creation_fields(peer, depth + 1))
        && children
            .iter()
            .all(|child| creation_fields(&child.creation, depth + 1))
}

fn created_field(step: &ApplicationOperationBodyCreatedField) -> bool {
    use ApplicationOperationBodyCreatedField as Step;
    match step {
        Step::Literal {
            field,
            binding: _,
            value: _,
        }
        | Step::Input {
            field,
            binding: _,
            scalar_family: _,
            member: _,
        }
        | Step::Computed {
            field,
            binding: _,
            value: _,
        }
        | Step::OptionalInput {
            field,
            binding: _,
            scalar_family: _,
            member: _,
        }
        | Step::SigningProvenance { field, binding: _ } => entity(field),
        Step::ScopeField {
            field,
            binding: _,
            scalar_family: _,
            source,
        }
        | Step::PresentScopeField {
            field,
            binding: _,
            scalar_family: _,
            source,
        }
        | Step::CapturedScopeField {
            field,
            binding: _,
            scalar_family: _,
            source,
            source_presence: _,
            role: _,
        }
        | Step::PrincipalIdentity {
            field,
            binding: _,
            scalar_family: _,
            mutation_binding: _,
            principal_binding: _,
            source,
        } => entity(field) && entity(source),
        Step::RelatedField {
            field,
            binding: _,
            scalar_family: _,
            source,
        } => entity(field) && entity(&source.field),
        Step::CapturedPathField {
            field,
            binding: _,
            scalar_family: _,
            source,
            role: _,
            target_presence: _,
        } => entity(field) && entity(&source.field),
    }
}

fn source_fields(source: &ApplicationOperationBodyRequirementSource) -> bool {
    use ApplicationOperationBodyRequirementSource as Source;
    match source {
        Source::Field {
            field,
            binding: _,
            scalar_family: _,
            presence: _,
            presence_only: _,
        }
        | Source::RelatedFieldValues {
            relation: _,
            from: _,
            to: _,
            field,
            binding: _,
            scalar_family: _,
            presence: _,
        }
        | Source::PrincipalIdentity {
            scope: _,
            mutation_binding: _,
            principal_binding: _,
            field,
            binding: _,
            scalar_family: _,
        } => entity(field),
        Source::SignedEvidence(source)
        | Source::RelatedSignedEvidence {
            relation: _,
            from: _,
            to: _,
            evidence: source,
        } => {
            let ApplicationSigningEvidenceSource {
                mutation_binding: _,
                operation: _,
                evidence_entity: _,
                provenance,
                purpose: _,
                producer_meaning: _,
                lifecycle,
            } = source;
            entity(provenance) && lifecycle_fields(lifecycle)
        }
        Source::QueryContext {
            mutation_binding: _,
            query: _,
        }
        | Source::RelatedItemCount {
            relation: _,
            from: _,
            to: _,
            direction: _,
        } => true,
    }
}

fn lifecycle_fields(lifecycle: &ApplicationOperationBodySigningLifecycleSpec) -> bool {
    let ApplicationOperationBodySigningLifecycleSpec {
        history_relation: _,
        current_relation: _,
        source_entity: _,
        consequence_relation: _,
        consequence_field,
        maximum_evidence: _,
    } = lifecycle;
    entity(consequence_field)
}
