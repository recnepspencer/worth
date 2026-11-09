mod symbol_text_lookup;
pub(crate) use symbol_text_lookup::SymbolTextLookup;
mod exhausted;
pub use exhausted::RelationalSymbolExhausted;
mod client_key;
mod client_key_symbol_policy;
mod interned_string;
mod string_interner;
mod symbol;
mod symbol_table_snapshot;

pub use client_key::ClientKey;
pub use client_key_symbol_policy::ClientKeySymbolPolicy;
pub use interned_string::InternedString;
pub use string_interner::StringInterner;
pub use symbol::Symbol;
pub use symbol_table_snapshot::SymbolTableSnapshot;

mod field_text;
mod stored_text;
pub use field_text::{RelationalFieldText, RelationalFieldTextAdmissionDenial};
pub use stored_text::StoredSymbolText;
pub(crate) use stored_text::SymbolTextLookupKey;

mod text_wire;
pub(crate) use text_wire::SymbolSnapshotWire;

pub(crate) use field_text::field_text_recovery_stop;

mod commit_texts;

pub(crate) use commit_texts::visit_required_commit_symbols;

#[cfg(test)]
pub(crate) use commit_texts::required_commit_symbols;

mod snapshot_observation;
pub use snapshot_observation::observe_symbol_snapshot_materialization;
