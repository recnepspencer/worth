fn main() {
    let _ = worth_foundational::FoundationalSelectedNodeScopeLocator::new(
        worth_foundational::FoundationalBranchId::new("feature").unwrap(),
        worth_foundational::FoundationalBranchId::new("main").unwrap(),
        "gear",
    );

    let _ = worth_foundational::FoundationalSelectedAspectScopeLocator::new(
        worth_foundational::FoundationalBranchId::new("feature").unwrap(),
        worth_foundational::FoundationalBranchId::new("main").unwrap(),
        "gear.teeth",
    );
}
