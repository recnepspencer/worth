//! One exact body-requirement record shared by schema and program identity.
use super::{
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementProgram,
    ApplicationOperationBodyRequirementSource as Source,
};
use std::fmt::Write;

impl ApplicationOperationBodyRequirementProgram {
    pub(crate) fn canonical_record(&self) -> String {
        let mut output = String::new();
        let mut part =
            |value: &str| write!(&mut output, "{}:{value}", value.len()).expect("String write");
        part("operation-body-requirement-v1");
        part(self.identity());
        part(self.target_name());
        part(&self.operands().len().to_string());
        for operand in self.operands() {
            part(operand.name());
            match operand.source() {
                Source::QueryContext {
                    mutation_binding,
                    query,
                } => {
                    part("observed-parameter-free-query-context-v1");
                    part(mutation_binding);
                    part(
                        &worth_foundational::facade::canonical_basis_sequence_material(
                            query.canonical_basis().basis().payload(),
                        ),
                    );
                }
                Source::RelatedFieldValues {
                    relation,
                    from,
                    to,
                    field,
                    binding,
                    scalar_family,
                    presence,
                } => {
                    part("distinct-live-outgoing-field-values-v1");
                    for text in [
                        relation.as_str(),
                        from.as_str(),
                        to.as_str(),
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        binding.as_str(),
                        scalar_family.canonical_name(),
                    ] {
                        part(text);
                    }
                    part(match presence {
                        crate::application_schema::ApplicationFieldPresence::Required => "required",
                        crate::application_schema::ApplicationFieldPresence::Optional => "optional",
                    });
                }
                Source::SignedEvidence(source) => {
                    part("current-signed-evidence");
                    part(&source.canonical_record());
                }
                Source::RelatedSignedEvidence {
                    relation,
                    from,
                    to,
                    evidence,
                } => {
                    part("related-current-signed-evidence-v1");
                    part(relation);
                    part(from);
                    part(to);
                    part(&evidence.canonical_record());
                }
                Source::Field {
                    field,
                    binding,
                    scalar_family,
                    presence,
                    presence_only,
                } => {
                    part("field");
                    part(field.entity());
                    part(field.aspect());
                    part(field.field());
                    part(binding.as_str());
                    part(scalar_family.canonical_name());
                    part(match presence {
                        crate::application_schema::ApplicationFieldPresence::Required => "required",
                        crate::application_schema::ApplicationFieldPresence::Optional => "optional",
                    });
                    part(if *presence_only { "presence" } else { "value" });
                }
                Source::RelatedItemCount {
                    relation,
                    from,
                    to,
                    direction,
                } => {
                    part("distinct-live-item-count");
                    part(relation);
                    part(from);
                    part(to);
                    part(match direction {
                        Direction::Outgoing => "outgoing",
                        Direction::Incoming => "incoming",
                    });
                }
                Source::PrincipalIdentity {
                    scope,
                    mutation_binding,
                    principal_binding,
                    field,
                    binding,
                    scalar_family,
                } => {
                    part("admitted-principal-identity");
                    for value in [
                        scope.as_str(),
                        mutation_binding.as_str(),
                        principal_binding.as_str(),
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        binding.as_str(),
                        scalar_family.canonical_name(),
                    ] {
                        part(value);
                    }
                }
            }
        }
        part(if self.applicability().is_some() {
            "conditional"
        } else {
            "always"
        });
        if let Some(expression) = self.applicability() {
            part(&hex(expression.draft()));
            part(expression.semantic_material());
        }
        part(&hex(self.satisfaction().draft()));
        part(self.satisfaction().semantic_material());
        output
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("String write");
    }
    output
}
