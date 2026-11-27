//! Shared pending text identity; candidate custody never publishes meaning.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, Weak};
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;

use super::RuntimeSymbolTable;
use crate::storage::data::RelationalStorageUnavailable;
use crate::symbols::data::{
    StoredSymbolText, StringInterner, Symbol, SymbolTableSnapshot, SymbolTextLookupKey,
};

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
pub enum SymbolReservationDenial {
    ColdBucket(StoredSymbolText),
    StorageUnavailable(RelationalStorageUnavailable),
    IdentityExhausted,
    InvalidDictionary,
}

/// One candidate's unpublished meanings and its repeatable dictionary view.
/// Reservations for equal text share an identity across concurrent candidates.
#[derive(Debug)]
pub struct PendingSymbolMeanings {
    owner: RuntimeSymbolTable,
    binding: RuntimeSymbolTable,
    interner: StringInterner,
    admitted: BTreeMap<Symbol, StoredSymbolText>,
    reserved: BTreeMap<Symbol, Arc<ReservedSymbolMeaning>>,
    required: BTreeSet<Symbol>,
}

/// Only meanings carried by the accepted canonical envelope may publish.
#[derive(Debug)]
pub struct PreparedSymbolPublication {
    owner: RuntimeSymbolTable,
    meanings: SymbolTableSnapshot,
    admissions: super::admission_transfer::PreparedSymbolAdmissionTransfer,
    reservations: Vec<Arc<ReservedSymbolMeaning>>,
}

impl PreparedSymbolPublication {
    pub fn publish(self) -> Result<(), &'static str> {
        self.owner.import_canonical_snapshot(&self.meanings)?;
        let result = self.admissions.publish(&self.owner);
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
        state.import_canonical_meaning(symbol, &text).map_err(|_| {
            crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary
        })?;
        drop(meaning);
        Ok(symbol)
    }

    pub fn begin_pending_meanings(&self) -> PendingSymbolMeanings {
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
    /// Admit the exact closed suffix's dictionary input before reservation.
    /// The pending candidate retains its existing identities; this performs no
    /// mutation, identity issuance or publication. Loads use the original request.
    pub fn demand_text(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        value: &str,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if self.interner.contains(value) {
            return Ok(());
        }
        self.owner.demand_text(execution, value)
    }

    /// A reserved dictionary identity is an actual candidate effect even if
    /// no application record survives in its authoritative patch.
    pub fn has_reserved_meanings(&self) -> bool {
        !self.reserved.is_empty()
    }
    /// A later count above an earlier one is a new candidate dictionary effect.
    pub fn reserved_meaning_count(&self) -> usize {
        self.reserved.len()
    }
    /// The same dictionary view under separate custody. Whatever a judgment
    /// reserves or requires in the copy is dropped with it, never published.
    pub fn discardable_copy(&self) -> Result<Self, SymbolReservationDenial> {
        let mut copy = self.owner.begin_pending_meanings();
        copy.adopt(self)?;
        Ok(copy)
    }

    pub fn reserve_revision_reference(
        &mut self,
        text: &str,
    ) -> Result<Symbol, SymbolReservationDenial> {
        let symbol = self.reserve(text)?;
        self.required.insert(symbol);
        Ok(symbol)
    }
    pub fn required_references(&self) -> &BTreeSet<Symbol> {
        &self.required
    }
    /// Forget every required revision reference, so a coupled candidate can
    /// require again exactly those of its surviving deltas (#119). Reserved
    /// meanings are kept; only the required set is rebuilt.
    pub fn clear_required_references(&mut self) {
        self.required.clear();
    }
    pub fn adopt(&mut self, prior: &Self) -> Result<(), SymbolReservationDenial> {
        if !Arc::ptr_eq(&self.owner.state, &prior.owner.state) {
            return Err(SymbolReservationDenial::InvalidDictionary);
        }
        for (symbol, text) in &prior.admitted {
            self.interner
                .import_canonical_meaning(*symbol, text)
                .map_err(|_| SymbolReservationDenial::InvalidDictionary)?;
            self.admitted.insert(*symbol, text.clone());
        }
        self.reserved.extend(
            prior
                .reserved
                .iter()
                .map(|(symbol, meaning)| (*symbol, meaning.clone())),
        );
        Ok(())
    }

    pub fn binding(&self) -> RuntimeSymbolTable {
        let binding = self.binding.detached_owner_snapshot();
        binding.with_write(|symbols| *symbols = self.interner.clone());
        binding.update(|state| {
            for (symbol, text) in &self.admitted {
                state
                    .configuration
                    .insert(SymbolTextLookupKey::from_text(text), *symbol);
            }
        });
        binding
    }

    /// Carry owner-issued admission and cold meanings from the immediately
    /// preceding applied pin. The caller verifies the proposal and revision.
    /// Existing admission transfer checks the exact cold directory and every
    /// carried meaning; only this new candidate binding receives the marks.
    pub fn binding_from_applied_predecessor(
        &self,
        previous: &RuntimeSymbolTable,
    ) -> Result<RuntimeSymbolTable, &'static str> {
        let binding = self.binding();
        previous
            .prepare_admission_transfer(&self.binding, &self.interner)?
            .publish(&binding)?;
        Ok(binding)
    }

    pub fn interner(&self) -> &StringInterner {
        &self.interner
    }

    /// No I/O occurs here. Closed input sets demand their exact text first.
    /// Otherwise a cold miss returns before issuing that text's identity, for
    /// the enclosing preparation owner to handle at its residency boundary.
    pub fn reserve(&mut self, value: &str) -> Result<Symbol, SymbolReservationDenial> {
        if let Some(symbol) = self.interner.symbol(value) {
            return Ok(symbol);
        }
        self.owner.require_resident_bucket(value)?;
        let mut state = self
            .owner
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let (symbol, text, reservation) = if let Some(symbol) = state.interner.symbol(value) {
            (
                symbol,
                state
                    .interner
                    .retained_text(symbol)
                    .ok_or(SymbolReservationDenial::InvalidDictionary)?
                    .clone(),
                None,
            )
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
                        .interner
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
            .import_canonical_meaning(symbol, &text)
            .map_err(|_| SymbolReservationDenial::InvalidDictionary)?;
        if let Some(reservation) = reservation {
            self.reserved.insert(symbol, reservation);
        }
        self.admitted.insert(symbol, text);
        Ok(symbol)
    }

    pub fn seal_for_publication(
        mut self,
        meanings: &SymbolTableSnapshot,
        binding: &RuntimeSymbolTable,
        required: impl IntoIterator<Item = Symbol>,
    ) -> Result<PreparedSymbolPublication, &'static str> {
        if meanings.entries.iter().any(|(symbol, text)| {
            symbol.0 == u32::MAX || self.interner.resolve(*symbol) != Some(text.as_str())
        }) {
            return Err("canonical text differs from candidate meaning");
        }
        let admissions = binding.prepare_admission_transfer(&self.binding, &self.interner)?;
        admissions.verify_required(required, binding, &self.owner, meanings)?;
        let reservations = meanings
            .entries
            .iter()
            .filter_map(|(symbol, _)| self.reserved.remove(symbol))
            .collect();
        Ok(PreparedSymbolPublication {
            owner: self.owner,
            meanings: meanings.clone(),
            admissions,
            reservations,
        })
    }
}

#[cfg(test)]
#[path = "reservations/tests.rs"]
mod tests;
