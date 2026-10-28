//! Native dictionary capacity is distinct from invalid text or missing meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalSymbolExhausted;
impl std::fmt::Display for RelationalSymbolExhausted {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("native symbol identity space exhausted")
    }
}
impl std::error::Error for RelationalSymbolExhausted {}
