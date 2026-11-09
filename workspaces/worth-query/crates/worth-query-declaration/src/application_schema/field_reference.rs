//! Entity marker names are derived from their top-down entity declarations.
pub trait ApplicationEntityMarkerIdentity<Schema> {
    const IDENTIFIER: &'static str;
}
