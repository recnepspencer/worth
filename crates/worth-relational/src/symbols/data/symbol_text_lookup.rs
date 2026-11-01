//! Pure borrowed symbol DATA used by root and storage reconstruction.
use super::{StoredSymbolText, StringInterner, Symbol};

pub(crate) trait SymbolTextLookup {
    fn symbol(&self, value: &str) -> Option<Symbol>;
    fn resolve(&self, symbol: Symbol) -> Option<&str>;
    fn retained_text(&self, symbol: Symbol) -> Option<&StoredSymbolText>;
}
impl SymbolTextLookup for StringInterner {
    fn symbol(&self, value: &str) -> Option<Symbol> {
        StringInterner::symbol(self, value)
    }
    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        StringInterner::resolve(self, symbol)
    }
    fn retained_text(&self, symbol: Symbol) -> Option<&StoredSymbolText> {
        StringInterner::retained_text(self, symbol)
    }
}
