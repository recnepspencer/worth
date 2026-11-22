/// How strictly identity lookups are verified while resolving a principal or entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionMode {
    /// Trust the identity index.
    Ordinary,
    /// Also verify each index lookup against stored records.
    Certification,
}
