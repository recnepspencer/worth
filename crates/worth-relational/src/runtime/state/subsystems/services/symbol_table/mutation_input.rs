//! The closed mutation symbol input: live pending reservation.
use super::{PendingSymbolMeanings, SymbolReservationDenial};
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
