use worth_foundational::{
    AspectKey, AspectMask, AspectMaskLocator, CanonicalFieldPath, DiagnosticMask, FieldKey,
    LocatorAuthority, MutationMask,
};

fn needs_diagnostic_locator(_locator: AspectMaskLocator<DiagnosticMask>) {}

fn main() {
    let mask = AspectMask::<MutationMask>::new([CanonicalFieldPath::single(
        FieldKey::new("done").unwrap(),
    )]);
    let locator = AspectMaskLocator::mutation(
        LocatorAuthority::Authoritative,
        AspectKey::new("task.summary").unwrap(),
        &mask,
    );

    needs_diagnostic_locator(locator);
}
