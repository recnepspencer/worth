//! Recorded owned-descriptor wire bytes, independent of the borrowed encoder.
use super::*;
use crate::identity::data::KindId;
use crate::validation::data::*;
struct RecordedRule {
    name: &'static str,
    version: CustomInvariantSemanticVersion,
    display: &'static str,
    work: Option<std::num::NonZeroU64>,
}
impl CustomInvariantRule for RecordedRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.name),
                semantic_version: self.version,
            },
            display_name: self.display.into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: self.work,
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(3)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![KindId(3)],
                    affected_relation_kinds: vec![KindId(2)],
                    include_relation_endpoint_entity_touches: true,
                },
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}
#[test]
fn borrowed_invariant_digest_preserves_recorded_owned_tuple_encoding_and_order() {
    // Recorded fields: alpha (2,7), UTF-8 display, work13; beta (1,4),
    // display B, no work ceiling. Both: commit, schema group, touched/block,
    // access [1,3]/[2]/[3]/[2]. The reference used explicit LE wire fields,
    // independent of either Rust tuple collector or this production encoder.
    let mut registrations = vec![
        CustomInvariantRegistration::new(RecordedRule {
            name: "wire.beta",
            version: CustomInvariantSemanticVersion::new(1, 4),
            display: "B",
            work: None,
        })
        .unwrap(),
        CustomInvariantRegistration::new(RecordedRule {
            name: "wire.alpha",
            version: CustomInvariantSemanticVersion::new(2, 7),
            display: "Alpha β",
            work: std::num::NonZeroU64::new(13),
        })
        .unwrap(),
    ];
    let render = |bytes: [u8; 32]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let expected = "73db667b8e83c4b812de0a8a222366b22101a30161d433a8b64dbc5bd102629d";
    assert_eq!(
        render(custom_invariant_inventory_digest(&registrations)),
        expected
    );
    registrations.reverse();
    assert_eq!(
        render(custom_invariant_inventory_digest(&registrations)),
        expected
    );
}
