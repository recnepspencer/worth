//! Borrowed native vocabulary: managed UTF-8 is admitted before its first owned copy.
use super::{RelationalFieldTextAdmissionDenial, StoredSymbolText, Symbol, SymbolTableSnapshot};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
#[derive(Deserialize)]
pub(crate) struct SymbolSnapshotWire<'a> {
    #[serde(borrow)]
    pub(crate) entries: Vec<(Symbol, SymbolTextWire<'a>)>,
}
pub(crate) enum SymbolTextWire<'a> {
    System(Cow<'a, str>),
    Managed(&'a str),
}
impl<'de: 'a, 'a> Deserialize<'de> for SymbolTextWire<'a> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Text<'a>(std::marker::PhantomData<&'a str>);
        impl<'de: 'a, 'a> serde::de::Visitor<'de> for Text<'a> {
            type Value = SymbolTextWire<'a>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("System text or borrowed managed_utf8_v1 text")
            }
            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                text: &'de str,
            ) -> Result<Self::Value, E> {
                Ok(SymbolTextWire::System(Cow::Borrowed(text)))
            }
            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<Self::Value, E> {
                Ok(SymbolTextWire::System(Cow::Owned(text.to_owned())))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let key = map
                    .next_key::<&'de str>()?
                    .ok_or_else(|| serde::de::Error::custom("missing managed text format"))?;
                if key != "managed_utf8_v1" {
                    return Err(serde::de::Error::custom("unsupported managed text format"));
                }
                // &str refuses an owned deserializer path. No transient String
                // for managed payloads is permitted before native admission.
                let text = map.next_value::<&'de str>()?;
                if map.next_key::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("extra managed text fields"));
                }
                Ok(SymbolTextWire::Managed(text))
            }
        }
        deserializer.deserialize_any(Text(std::marker::PhantomData))
    }
}
impl SymbolSnapshotWire<'_> {
    pub(crate) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SymbolTableSnapshot, RelationalFieldTextAdmissionDenial> {
        let mut ids = std::collections::BTreeMap::new();
        let mut values = std::collections::BTreeMap::new();
        for (symbol, text) in &self.entries {
            execution
                .check_live()
                .map_err(RelationalFieldTextAdmissionDenial::Stopped)?;
            let raw = match text {
                SymbolTextWire::System(text) => text.as_ref(),
                SymbolTextWire::Managed(text) => text,
            };
            if symbol.0 == 0
                || symbol.0 == u32::MAX
                || ids.insert(*symbol, raw).is_some_and(|prior| prior != raw)
                || values
                    .insert(raw, *symbol)
                    .is_some_and(|prior| prior != *symbol)
            {
                return Err(RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary);
            }
        }
        drop((ids, values));
        let mut entries = Vec::new();
        for (symbol, text) in self.entries {
            execution
                .check_live()
                .map_err(RelationalFieldTextAdmissionDenial::Stopped)?;
            let text = match text {
                SymbolTextWire::System(text) => StoredSymbolText::from(text.into_owned()),
                SymbolTextWire::Managed(text) => StoredSymbolText::copy_from_str(execution, text)?,
            };
            entries.push((symbol, text));
        }
        Ok(SymbolTableSnapshot { entries })
    }
}
