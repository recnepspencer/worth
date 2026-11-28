//! Borrow the exact dictionary entries on each serializer pass; no tuple Vec.
use crate::durability::log::native_checkpoint_codec::member_frame;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::symbols::data::stored_text::StoredSymbolText;
use contracts::symbols::data::Symbol;
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;

pub(in crate::internal_port::durability::authority::checkpoint_complete) fn encode_dictionary<
    'a,
    I: Iterator<Item = (Symbol, &'a StoredSymbolText)>,
>(
    count: usize,
    entries: impl Fn() -> I,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalDurableMemberLocator, ExecutionImmutableBytes), RelationalRecoveryStop> {
    member_frame::encode_for_request(MemberKind::Symbols, &view(count, entries), request)
}
pub(super) fn view<'a, I: Iterator<Item = (Symbol, &'a StoredSymbolText)>>(
    count: usize,
    entries: impl Fn() -> I,
) -> impl Serialize {
    Dictionary { count, entries }
}
struct Dictionary<F> {
    count: usize,
    entries: F,
}
impl<'a, I: Iterator<Item = (Symbol, &'a StoredSymbolText)>, F: Fn() -> I> Serialize
    for Dictionary<F>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut row = serializer.serialize_struct("SymbolTableSnapshot", 1)?;
        row.serialize_field("entries", &Entries(self))?;
        row.end()
    }
}
struct Entries<'a, F>(&'a Dictionary<F>);
impl<'a, I: Iterator<Item = (Symbol, &'a StoredSymbolText)>, F: Fn() -> I> Serialize
    for Entries<'_, F>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.count))?;
        let mut count = 0usize;
        for entry in (self.0.entries)() {
            if count == self.0.count {
                return Err(serde::ser::Error::custom("dictionary count changed"));
            }
            seq.serialize_element(&entry)?;
            count += 1;
        }
        if count != self.0.count {
            return Err(serde::ser::Error::custom("dictionary count changed"));
        }
        seq.end()
    }
}
