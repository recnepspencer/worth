//! Shared pending text identity; candidate custody never publishes meaning.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, Weak};

use super::RuntimeSymbolTable;
use crate::storage::data::RelationalStorageUnavailable;
use crate::symbols::data::{StoredSymbolText, StringInterner, Symbol, SymbolTableSnapshot};

#[derive(Debug, Default)]
pub(super) struct SymbolReservationDirectory {
    pending: BTreeMap<StoredSymbolText, Weak<ReservedSymbolMeaning>>,
}

#[derive(Debug)]
struct ReservedSymbolMeaning {
    symbol: Symbol,
    text: StoredSymbolText,
    directory: Weak<Mutex<SymbolReservationDirectory>>,
}

impl Drop for ReservedSymbolMeaning {
    fn drop(&mut self) {
        let Some(directory) = self.directory.upgrade() else {
            return;
        };
        let mut directory = directory.lock().unwrap_or_else(|error| error.into_inner());
        if directory
            .pending
            .get(self.text.as_str())
            .is_some_and(|entry| std::ptr::eq(entry.as_ptr(), self))
        {
            directory.pending.remove(self.text.as_str());
        }
    }
}

#[derive(Debug)]
pub(crate) enum SymbolReservationDenial {
    ColdBucket(StoredSymbolText),
    StorageUnavailable(RelationalStorageUnavailable),
    IdentityExhausted,
}

/// One candidate's unpublished meanings and its repeatable dictionary view.
/// Reservations for equal text share an identity across concurrent candidates.
#[derive(Debug)]
pub(crate) struct PendingSymbolMeanings {
    owner: RuntimeSymbolTable,
    binding: RuntimeSymbolTable,
    interner: StringInterner,
    admitted: BTreeMap<Symbol, StoredSymbolText>,
    reserved: BTreeMap<Symbol, Arc<ReservedSymbolMeaning>>,
    required: BTreeSet<Symbol>,
}

/// Only meanings carried by the accepted canonical envelope may publish.
#[derive(Debug)]
pub(crate) struct PreparedSymbolPublication {
    owner: RuntimeSymbolTable,
    meanings: SymbolTableSnapshot,
    reservations: Vec<Arc<ReservedSymbolMeaning>>,
}

impl PreparedSymbolPublication {
    pub(crate) fn publish(self) -> Result<(), &'static str> {
        let result = self.owner.import_canonical_snapshot(&self.meanings);
        drop(self.reservations);
        result
    }
}

impl RuntimeSymbolTable {
    pub(super) fn pending_symbol(&self, value: &str) -> Option<Symbol> {
        self.pending_meaning(value).map(|meaning| meaning.symbol)
    }

    fn pending_meaning(&self, value: &str) -> Option<Arc<ReservedSymbolMeaning>> {
        let directory = self
            .reservations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let meaning = directory.pending.get(value).and_then(Weak::upgrade);
        drop(directory);
        meaning
    }

    pub(super) fn admit_with_pending_identity(
        &self,
        state: &mut crate::symbols::data::StringInterner,
        text: StoredSymbolText,
    ) -> Result<Symbol, crate::symbols::data::RelationalFieldTextAdmissionDenial> {
        let Some(meaning) = self.pending_meaning(text.as_str()) else {
            return state.admit_text(text);
        };
        let symbol = meaning.symbol;
        state
            .import_canonical_snapshot(&SymbolTableSnapshot {
                entries: vec![(symbol, text)],
            })
            .map_err(|_| {
                crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary
            })?;
        drop(meaning);
        Ok(symbol)
    }

    pub(crate) fn begin_pending_meanings(&self) -> PendingSymbolMeanings {
        let binding = self.detached_owner_snapshot();
        PendingSymbolMeanings {
            owner: self.clone(),
            interner: binding.interner_snapshot(),
            binding,
            admitted: BTreeMap::new(),
            reserved: BTreeMap::new(),
            required: BTreeSet::new(),
        }
    }

    fn require_resident_bucket(&self, text: &str) -> Result<(), SymbolReservationDenial> {
        if self.with_read(|symbols| symbols.contains(text)) {
            return Ok(());
        }
        let cold = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some(cold) = cold {
            match cold.value_bucket_is_resident(text) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(SymbolReservationDenial::ColdBucket(text.to_owned().into()))
                }
                Err(failure) => return Err(SymbolReservationDenial::StorageUnavailable(failure)),
            }
        }
        Ok(())
    }
}

impl PendingSymbolMeanings {
    pub(crate) fn reserve_revision_reference(
        &mut self,
        text: &str,
    ) -> Result<Symbol, SymbolReservationDenial> {
        let symbol = self.reserve(text)?;
        self.required.insert(symbol);
        Ok(symbol)
    }
    pub(crate) fn required_references(&self) -> &BTreeSet<Symbol> {
        &self.required
    }
    pub(crate) fn adopt(&mut self, prior: &Self) {
        assert!(
            Arc::ptr_eq(&self.owner.state, &prior.owner.state),
            "pending meanings belong to one runtime owner"
        );
        for (symbol, text) in &prior.admitted {
            self.interner
                .import_canonical_snapshot(&SymbolTableSnapshot {
                    entries: vec![(*symbol, text.clone())],
                })
                .expect("same-owner pending meanings cannot alias");
            self.admitted.insert(*symbol, text.clone());
        }
        self.reserved.extend(
            prior
                .reserved
                .iter()
                .map(|(symbol, meaning)| (*symbol, meaning.clone())),
        );
    }

    pub(crate) fn binding(&self) -> RuntimeSymbolTable {
        let binding = self.binding.detached_owner_snapshot();
        binding.with_write(|symbols| *symbols = self.interner.clone());
        let mut configuration = binding
            .configuration_snapshot
            .write()
            .unwrap_or_else(|error| error.into_inner());
        for (symbol, text) in &self.admitted {
            configuration.insert(text.clone(), *symbol);
        }
        drop(configuration);
        binding
    }

    pub(crate) fn interner(&self) -> &StringInterner {
        &self.interner
    }

    /// No I/O occurs here. A cold bucket is a discovery for the enclosing
    /// candidate round, which must abandon this custody before demanding it.
    pub(crate) fn reserve(&mut self, value: &str) -> Result<Symbol, SymbolReservationDenial> {
        if let Some(symbol) = self.interner.symbol(value) {
            return Ok(symbol);
        }
        self.owner.require_resident_bucket(value)?;
        let mut state = self
            .owner
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let (symbol, text, reservation) = if let Some(symbol) = state.symbol(value) {
            (symbol, state.retained_text(symbol).unwrap().clone(), None)
        } else {
            let mut directory = self
                .owner
                .reservations
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let reservation = match directory.pending.get(value).and_then(Weak::upgrade) {
                Some(reservation) => reservation,
                None => {
                    let symbol = state
                        .reserve_identity()
                        .map_err(|_| SymbolReservationDenial::IdentityExhausted)?;
                    let reservation = Arc::new(ReservedSymbolMeaning {
                        symbol,
                        text: value.to_owned().into(),
                        directory: Arc::downgrade(&self.owner.reservations),
                    });
                    directory
                        .pending
                        .insert(reservation.text.clone(), Arc::downgrade(&reservation));
                    reservation
                }
            };
            (
                reservation.symbol,
                reservation.text.clone(),
                Some(reservation),
            )
        };
        drop(state);
        self.interner
            .import_canonical_snapshot(&SymbolTableSnapshot {
                entries: vec![(symbol, text.clone())],
            })
            .expect("candidate meaning comes from the shared owner reservation");
        if let Some(reservation) = reservation {
            self.reserved.insert(symbol, reservation);
        }
        self.admitted.insert(symbol, text);
        Ok(symbol)
    }

    pub(crate) fn seal_for_publication(
        mut self,
        meanings: &SymbolTableSnapshot,
    ) -> PreparedSymbolPublication {
        let reservations = meanings
            .entries
            .iter()
            .filter_map(|(symbol, text)| {
                assert_eq!(
                    self.interner.resolve(*symbol),
                    Some(text.as_str()),
                    "canonical text comes from candidate meaning"
                );
                self.reserved.remove(symbol)
            })
            .collect();
        PreparedSymbolPublication {
            owner: self.owner,
            meanings: meanings.clone(),
            reservations,
        }
    }
}

#[cfg(test)]
#[path = "reservations/tests.rs"]
mod tests;
