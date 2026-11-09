//! Closed mutation symbol inputs: live reservation or authenticated original DATA.
use super::{PendingSymbolMeanings, SymbolReservationDenial};
#[cfg(test)]
use crate::symbols::data::SymbolTableSnapshot;
use crate::symbols::data::{Symbol, SymbolTextLookup};

mod sealed {
    pub trait Sealed {}
}

pub(crate) trait MutationSymbolMeanings: sealed::Sealed {
    fn reserve_revision_reference(&mut self, text: &str)
        -> Result<Symbol, SymbolReservationDenial>;
    fn lookup(&self) -> &dyn SymbolTextLookup;
    fn symbol(&self, text: &str) -> Option<Symbol> {
        self.lookup().symbol(text)
    }
}
impl sealed::Sealed for PendingSymbolMeanings {}
impl MutationSymbolMeanings for PendingSymbolMeanings {
    fn reserve_revision_reference(
        &mut self,
        text: &str,
    ) -> Result<Symbol, SymbolReservationDenial> {
        PendingSymbolMeanings::reserve_revision_reference(self, text)
    }
    fn lookup(&self) -> &dyn SymbolTextLookup {
        PendingSymbolMeanings::interner(self)
    }
}

/// Borrows the admitted canonical dictionary and reference roster. It cannot
/// reserve an identity, import an interner or allocate a reservation directory.
#[cfg(test)]
pub(crate) struct OriginalSymbolMeanings<'a> {
    lookup: &'a dyn SymbolTextLookup,
    references: &'a [Symbol],
}
#[cfg(test)]
impl<'a> OriginalSymbolMeanings<'a> {
    pub(crate) fn new(
        lookup: &'a dyn SymbolTextLookup,
        dictionary: &'a SymbolTableSnapshot,
        references: &'a [Symbol],
    ) -> Result<Self, SymbolReservationDenial> {
        if references.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(SymbolReservationDenial::InvalidDictionary);
        }
        for (symbol, text) in &dictionary.entries {
            if symbol.0 == u32::MAX
                || lookup.resolve(*symbol) != Some(text.as_str())
                || lookup.symbol(text.as_str()) != Some(*symbol)
            {
                return Err(SymbolReservationDenial::InvalidDictionary);
            }
        }
        Ok(Self { lookup, references })
    }
}
#[cfg(test)]
impl sealed::Sealed for OriginalSymbolMeanings<'_> {}
#[cfg(test)]
impl MutationSymbolMeanings for OriginalSymbolMeanings<'_> {
    fn reserve_revision_reference(
        &mut self,
        text: &str,
    ) -> Result<Symbol, SymbolReservationDenial> {
        self.symbol(text)
            .filter(|symbol| self.references.binary_search(symbol).is_ok())
            .ok_or(SymbolReservationDenial::InvalidDictionary)
    }
    fn lookup(&self) -> &dyn SymbolTextLookup {
        self.lookup
    }
    fn symbol(&self, text: &str) -> Option<Symbol> {
        self.lookup.symbol(text)
    }
}

#[cfg(test)]
mod tests;
