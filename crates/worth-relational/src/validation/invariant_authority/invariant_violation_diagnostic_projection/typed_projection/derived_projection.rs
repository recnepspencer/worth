//! Derived diagnostics preserve logical coordinates and real physical support.
use crate::derived_relation::{DerivedRelationIdentity, DerivedRelationLawViolation};
use crate::diagnostics::data::RelationalDiagnosticValue as Value;
use crate::graph_member::NativeMemberKey;

pub(super) fn graph(
    definition: &DerivedRelationIdentity,
    violation: &DerivedRelationLawViolation,
) -> Value {
    super::super::violation_diagnostic_object(
        "derived_relation_graph",
        [
            (
                "definition",
                Value::object([
                    ("schema", Value::string(&definition.schema.0)),
                    ("version", Value::Unsigned(u64::from(definition.version.0))),
                    ("name", Value::string(&definition.name)),
                ]),
            ),
            ("law", Value::string(violation.law.as_str())),
            ("member", member(&violation.member)),
            (
                "counterpart",
                violation.counterpart.as_ref().map_or(Value::Null, member),
            ),
            ("count", Value::Unsigned(violation.count)),
            ("limit", Value::Unsigned(violation.limit)),
            (
                "physical_entities",
                Value::array(
                    violation
                        .physical_entities
                        .iter()
                        .copied()
                        .map(Value::EntityId),
                ),
            ),
            (
                "physical_relations",
                Value::array(
                    violation
                        .physical_relations
                        .iter()
                        .copied()
                        .map(Value::RelationId),
                ),
            ),
        ],
    )
}

fn member(key: &NativeMemberKey) -> Value {
    match key {
        NativeMemberKey::Entity(entity) => Value::object([
            ("member_kind", Value::string("entity")),
            ("entity", Value::EntityId(*entity)),
        ]),
        NativeMemberKey::Projected(member) => Value::object([
            ("member_kind", Value::string("projected")),
            ("schema", Value::string(&member.domain.schema.0)),
            (
                "version",
                Value::Unsigned(u64::from(member.domain.version.0)),
            ),
            ("domain", Value::string(&member.domain.name)),
            ("carrier", Value::EntityId(member.carrier)),
            ("case", Value::string(&member.case)),
        ]),
    }
}
