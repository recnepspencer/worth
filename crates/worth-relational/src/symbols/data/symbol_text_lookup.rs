//! Pure borrowed symbol DATA used by root and storage reconstruction.
use super::{StringInterner, Symbol};

pub(crate) trait SymbolTextLookup {
    fn symbol(&self, value: &str) -> Option<Symbol>;
    fn resolve(&self, symbol: Symbol) -> Option<&str>;
}
impl SymbolTextLookup for StringInterner {
    fn symbol(&self, value: &str) -> Option<Symbol> {
        StringInterner::symbol(self, value)
    }
    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        StringInterner::resolve(self, symbol)
    }
}
