use worth_foundational::FoundationalBranchTarget;

use super::CANONICAL_RUNTIME_INSTANCE_ID;
use crate::branch::{
    RelationalBranchReferenceCell, RelationalBranchRootDescriptor, RelationalBranchTarget,
};
use crate::history::data::BranchId;

fn cell_on(runtime_instance_id: u64) -> RelationalBranchReferenceCell {
    let cell =
        RelationalBranchReferenceCell::empty(runtime_instance_id, BranchId("main".to_owned()))
            .expect("test branch identity is valid");
    cell.advance_truth(FoundationalBranchTarget::basis(
        RelationalBranchTarget::new(
            runtime_instance_id,
            4,
            3,
            vec![2],
            RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
        ),
    ))
    .expect("truth movement advances");
    cell
}

#[test]
fn equal_cells_of_two_runtimes_record_one_canonical_image() {
    let first = cell_on(1_000_000_007).checkpoint();
    let second = cell_on(1_000_000_008).checkpoint();
    assert_ne!(first, second, "live images are runtime-affine");
    let canonical = first.in_canonical_frame().expect("live image rebinds");
    assert_eq!(
        canonical,
        second.in_canonical_frame().expect("live image rebinds")
    );
    assert_eq!(canonical.runtime_instance_id, CANONICAL_RUNTIME_INSTANCE_ID);
    assert_eq!(
        canonical.observation.branch_id().as_str(),
        format!("relational/{CANONICAL_RUNTIME_INSTANCE_ID}/main")
    );
    assert_eq!(
        canonical.observation.branch_id().as_str().len(),
        first.observation.branch_id().as_str().len(),
        "recorded branch text keeps the live width"
    );
}

#[test]
fn a_canonical_image_rebinds_to_the_recovering_runtime() {
    let live = cell_on(1_000_000_007);
    let canonical = live
        .checkpoint()
        .in_canonical_frame()
        .expect("live image rebinds");
    let recovered =
        RelationalBranchReferenceCell::from_checkpoint_with_root(1_000_000_009, canonical, None)
            .expect("a canonical image is internally consistent");
    let rebound = live
        .rebind_runtime(1_000_000_009)
        .expect("live cell rebinds");
    assert_eq!(recovered.checkpoint(), rebound.checkpoint());
    assert_eq!(
        recovered.checkpoint().in_canonical_frame(),
        live.checkpoint().in_canonical_frame(),
        "rebinding is lossless"
    );
}
