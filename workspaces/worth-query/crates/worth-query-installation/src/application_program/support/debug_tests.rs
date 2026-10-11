use std::any::TypeId;
use std::collections::{BTreeSet, HashSet};

use super::super::program_support_fixture::{validated, BoundedProgram};
use super::WorthQueryProgramSupportEntry;

#[test]
fn debug_reports_inventory_sizes_without_type_identity() {
    let program = validated::<BoundedProgram>();
    let declarations = [TypeId::of::<u8>(), TypeId::of::<u16>()];
    let entry = |types: [TypeId; 2]| {
        WorthQueryProgramSupportEntry::admitted(
            *program.revision(),
            program.identity().clone(),
            BTreeSet::new(),
            HashSet::from(types),
            HashSet::from(types),
            program.semantic_description().clone(),
            BTreeSet::new(),
            BTreeSet::new(),
        )
    };
    let forward = format!("{:?}", entry(declarations));
    let reverse = format!("{:?}", entry([declarations[1], declarations[0]]));
    assert!(!forward.contains("TypeId"), "{forward}");
    assert!(!reverse.contains("TypeId"), "{reverse}");
    assert_eq!(forward, reverse);
    assert_eq!(
        forward,
        r#"WorthQueryProgramSupportEntry { revision: ApplicationProgramRevision([216, 129, 199, 205, 10, 237, 254, 231, 57, 182, 3, 18, 239, 177, 230, 245, 6, 189, 193, 85, 72, 77, 63, 16, 63, 34, 187, 252, 40, 167, 63, 119]), identity: ApplicationProgramIdentity("worth.query.installation-test.bounded-program.v1"), rules: [], action_operations: 2, mutation_bindings: 2, semantic_description: ApplicationSemanticDescription { revision: ApplicationProgramRevision([216, 129, 199, 205, 10, 237, 254, 231, 57, 182, 3, 18, 239, 177, 230, 245, 6, 189, 193, 85, 72, 77, 63, 16, 63, 34, 187, 252, 40, 167, 63, 119]), facts: [ApplicationSemanticFact { family: Features, subject: "4:root48:worth.query.installation-test.bounded-feature.v1", meaning: "major=1:1|minor=1:0" }, ApplicationSemanticFact { family: Rules, subject: "4:root16:BoundedDimension", meaning: "major=1:1|minor=1:0|point=14:CommitBoundary|owner=0:" }, ApplicationSemanticFact { family: Operations, subject: "4:root48:worth.query.installation-test.bounded-feature.v115:AdjustDimension", meaning: "input=46:worth.query.installation-test.adjust-dimension|conditional=4:true|required-output=5:false|invariant=0:|observation=0:|locality=0:|granule=0:|change=0:|posture=0:" }] }, effectful_action_subjects: [], action_dependencies: [] }"#
    );
}
