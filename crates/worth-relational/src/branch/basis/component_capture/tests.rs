//! Descriptive wire ordering only; no source, root or allocation authority.
use super::*;
#[test]
fn canonical_component_order_matches_original_named_messagepack_keys() {
    let mut sources = Vec::new();
    for name in [
        "z".to_owned(),
        "aa".to_owned(),
        "x".repeat(31),
        "x".repeat(32),
        "y".repeat(255),
        "y".repeat(256),
    ] {
        for (index, commit) in [
            0,
            1,
            127,
            128,
            255,
            256,
            65535,
            65536,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
        ]
        .into_iter()
        .enumerate()
        {
            let target = if index == 0 {
                FoundationalBranchTarget::Empty
            } else {
                FoundationalBranchTarget::Basis(RelationalBranchTarget::new(
                    7,
                    commit,
                    commit,
                    vec![
                        commit;
                        if index == 8 {
                            15
                        } else if index == 9 {
                            16
                        } else {
                            index
                        }
                    ],
                    RelationalBranchRootDescriptor::new([index as u8; 32], [9; 32]),
                ))
            };
            let reference = super::super::super::relational_branch_observation(
                7,
                &name,
                target,
                worth_foundational::FoundationalBranchReferenceGeneration::new(commit),
            )
            .unwrap();
            sources.push((
                BranchId(name.clone()),
                reference,
                RelationalBranchVersion::new(commit),
            ));
        }
    }
    let views = sources
        .iter()
        .enumerate()
        .map(
            |(index, (branch, reference, truth))| CheckpointComponentDescriptorView {
                branch,
                reference,
                truth: *truth,
                schema: [index as u8; 32],
                visibility: [3; 32],
                posture: match index % 3 {
                    0 => RelationalBranchBasisPosture::Live,
                    1 => RelationalBranchBasisPosture::Archived,
                    _ => RelationalBranchBasisPosture::Deleting,
                },
            },
        )
        .collect::<Vec<_>>();
    let mut expected = views
        .iter()
        .enumerate()
        .map(|(index, view)| {
            (
                rmp_serde::to_vec_named(&view.copy_canonical().unwrap()).unwrap(),
                index,
            )
        })
        .collect::<Vec<_>>();
    expected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    let mut actual = (0..views.len()).collect::<Vec<_>>();
    actual.sort_unstable_by(|left, right| views[*left].canonical_wire_cmp(&views[*right]));
    assert_eq!(
        actual,
        expected.iter().map(|(_, index)| *index).collect::<Vec<_>>()
    );
}
