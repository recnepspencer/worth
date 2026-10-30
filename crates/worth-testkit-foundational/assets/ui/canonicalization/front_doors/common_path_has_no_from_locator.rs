use worth_foundational::{
    canonicalization, AspectLocator, BoundarySourceLocator, CanonicalLocatorInput,
    CanonicalizationRuleVersion, LocatorAuthority,
};

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.front-door").expect("valid version");
    let locator = CanonicalLocatorInput::Source(BoundarySourceLocator::aspect(AspectLocator::new(
        LocatorAuthority::SupportOnly,
        worth_foundational::AspectKey::new("task.count").expect("valid key"),
    )));

    let _ = canonicalization().basis().at(version).from_locator(locator);
}
