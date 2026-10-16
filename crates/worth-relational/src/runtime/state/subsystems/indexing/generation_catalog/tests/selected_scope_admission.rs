//! Logical payload work belongs only to the selected index's branch scopes.
use super::*;
use crate::indexes::data::{SelectedIndexGenerationAdmissionStop as Stop, SelectedIndexReadWork};

// This is the unchanged ordinary adoption caller's selection allowance.
const SELECTION_WORK: u64 = 64;

fn current(
    catalog: &GenerationCatalog,
    branch: &BranchId,
) -> Result<(bool, u64, u64), Stop<&'static str>> {
    let mut logical = 0_u64;
    let mut bytes = 0_u64;
    let answer = catalog.has_published_for_commit_admitted(
        DerivedIndexId(1),
        Some(branch),
        CommitId(101),
        VersionId(1),
        &mut |work, scratch| {
            let units = match work {
                SelectedIndexReadWork::Operation(units) => units,
                SelectedIndexReadWork::OrderedNavigation(units) => {
                    assert!(units <= SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK);
                    1
                }
            };
            if units > SELECTION_WORK - logical {
                return Err("original selection work");
            }
            logical += units;
            bytes += scratch;
            Ok(())
        },
    )?;
    Ok((answer, logical, bytes))
}

#[test]
fn selected_scope_payload_and_currency_ignore_unrelated_indexes() {
    let mut catalog = GenerationCatalog::default();
    catalog.insert(generation(1, 1, 0, 1, 1, true));
    let branch = BranchId("branch-0".into());
    let baseline = current(&catalog, &branch).unwrap();
    assert!(baseline.0);
    for index in 2..258 {
        catalog.insert(generation(index, index, 0, 1, 1, true));
    }
    assert_eq!(current(&catalog, &branch).unwrap(), baseline);
    assert!(!current(&catalog, &BranchId("absent".into())).unwrap().0);
    let refused = catalog.has_published_for_commit_admitted(
        DerivedIndexId(1),
        Some(&branch),
        CommitId(101),
        VersionId(1),
        &mut |_, _| Err("original refusal"),
    );
    assert!(matches!(refused, Err(Stop::Admission("original refusal"))));
    assert_eq!(current(&catalog, &branch).unwrap(), baseline);
    assert_eq!(
        id(catalog.exact(
            DerivedIndexId(1),
            Some(&branch),
            VersionId(1),
            SchemaVersionId(1)
        )),
        Some(DerivedIndexGenerationId(1))
    );
}

#[test]
fn selected_branch_payload_still_spends_the_original_allowance() {
    let mut catalog = GenerationCatalog::default();
    let branch = BranchId("x".repeat(128));
    let mut selected = generation(1, 1, 0, 1, 1, true);
    selected.source_branch_id = branch.clone();
    selected.applicability.branch_id = branch.clone();
    catalog.insert(selected);
    assert!(matches!(
        current(&catalog, &branch),
        Err(Stop::Admission("original selection work"))
    ));
    assert!(catalog.scope(DerivedIndexId(1), Some(&branch)).is_some());
}
