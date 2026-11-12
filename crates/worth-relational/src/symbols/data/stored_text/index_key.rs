//! Compact ordered lookup retaining the text's actual immutable byte owner.
use super::{Backing, StoredSymbolText};
use std::{borrow::Borrow, cmp::Ordering, sync::Arc};
use worth_execution::ExecutionImmutableBytes;

#[derive(Clone, Debug)]
pub(crate) struct SymbolTextLookupKey {
    bytes: ExecutionImmutableBytes,
}
// The dictionary's recursive nodes carry a byte handle, never a second text carrier.
const _: () =
    assert!(std::mem::size_of::<SymbolTextLookupKey>() <= 2 * std::mem::size_of::<usize>());

impl SymbolTextLookupKey {
    pub(crate) fn from_owned(text: StoredSymbolText) -> Self {
        let bytes = match text.backing {
            Backing::System(text) => ExecutionImmutableBytes::from_external_bytes(Arc::new(
                text.into_bytes().into_boxed_slice(),
            )),
            Backing::Managed(bytes) => bytes,
        };
        Self { bytes }
    }
    pub(crate) fn from_text(text: &StoredSymbolText) -> Self {
        Self::from_owned(text.clone())
    }
    pub(crate) fn as_str(&self) -> &str {
        std::str::from_utf8(self.bytes.bytes())
            .expect("lookup keys are built only from UTF-8 StoredSymbolText")
    }
    pub(crate) fn is_managed(&self) -> bool {
        !self.bytes.is_external_backing()
    }
    pub(crate) fn retained_text(&self) -> StoredSymbolText {
        if self.bytes.is_external_backing() {
            StoredSymbolText::from(self.as_str())
        } else {
            StoredSymbolText {
                backing: Backing::Managed(self.bytes.clone()),
            }
        }
    }
}
impl Borrow<str> for SymbolTextLookupKey {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
impl PartialEq for SymbolTextLookupKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for SymbolTextLookupKey {}
impl PartialOrd for SymbolTextLookupKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SymbolTextLookupKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}
