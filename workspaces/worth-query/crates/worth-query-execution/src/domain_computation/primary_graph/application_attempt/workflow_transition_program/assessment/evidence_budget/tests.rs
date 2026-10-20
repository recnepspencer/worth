use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCreationPartition;
use crate::domain_computation::primary_graph::tests::fixture::installed_layout;

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

fn evidence_effects(
    layout: &WorthQueryWorkflowLayout,
    publication: &str,
) -> [WorthQueryApplicationRealizedEffect; 2] {
    let entity = |kind, fields| WorthQueryApplicationRealizedEffect::CreateEntity {
        kind,
        key: "evidence-width".to_owned(),
        fields,
        partition: WorthQueryApplicationCreationPartition::Issued,
    };
    [
        entity(
            layout.assessment_evidence.entity_kind,
            BTreeMap::from([
                (
                    layout.assessment_evidence.publication_identity.clone(),
                    text(publication),
                ),
                (
                    layout.assessment_evidence.retained_bytes.clone(),
                    AspectValue::UInt64(0),
                ),
            ]),
        ),
        entity(
            layout.evidence_dependency.entity_kind,
            BTreeMap::from([
                (layout.evidence_dependency.fact_kind.clone(), text("field")),
                (
                    layout.evidence_dependency.native_revision.clone(),
                    AspectValue::UInt64(3),
                ),
                (
                    layout.evidence_dependency.field_presence.clone(),
                    AspectValue::Bool(true),
                ),
            ]),
        ),
    ]
}

#[test]
fn inclusive_evidence_admission_charges_the_decimal_publication_width() {
    let primary = installed_layout();
    let layout = primary.workflow();
    // Literal text is the width subject, not manufactured runtime authority.
    // Field tags contribute one byte each; u64 is nine and bool is two.
    for (publication, charge) in [
        ("9:main:1:1:1:1:1:1:1", 21 + 9 + 6 + 9 + 2),
        ("10:main:1:1:1:1:1:1:1", 22 + 9 + 6 + 9 + 2),
    ] {
        for allowance in [charge - 1, charge, charge + 1] {
            let result = admit(layout, allowance, "checks/width", |observe| {
                for effect in evidence_effects(layout, publication) {
                    observe(effect)?;
                }
                Ok(())
            });
            if allowance < charge {
                let denial = result.err().expect("one missing byte refuses evidence");
                assert_eq!(
                    denial.kind(),
                    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceEvidenceCapacityUnavailable,
                );
            } else {
                let actual = result.expect("the inclusive evidence allowance is sufficient");
                assert_eq!(actual, charge);
            }
        }
    }
}
