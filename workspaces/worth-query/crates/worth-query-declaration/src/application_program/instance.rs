/// Stable authored path of one composition instance within an application root.
///
/// The path qualifies declaration meaning. It does not identify or allocate a
/// live domain occurrence.
pub trait ApplicationCompositionInstance: Sized + 'static {
    const PATH: &'static str;
}

/// The application root used by declarations that are not nested.
pub struct ApplicationRootComposition;

impl ApplicationCompositionInstance for ApplicationRootComposition {
    const PATH: &'static str = "root";
}
