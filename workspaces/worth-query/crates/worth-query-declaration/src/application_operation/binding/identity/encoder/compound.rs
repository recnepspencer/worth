//! Compound values in progress: ordered members stream through, map entries wait.

use serde::ser::{self, Serialize};

use super::{encode_entry, encode_into, put_text, CanonicalEncodeError, Sink, END, MORE};

/// One compound value in progress. Ordered members stream straight through;
/// map entries wait in `entries` until `finish` sorts them.
pub(super) struct Compound<'a, S> {
    sink: &'a mut S,
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    pending_key: Option<Vec<u8>>,
}

impl<'a, S: Sink> Compound<'a, S> {
    pub(super) fn new(sink: &'a mut S) -> Self {
        Self {
            sink,
            entries: Vec::new(),
            pending_key: None,
        }
    }

    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CanonicalEncodeError> {
        self.sink.put(&[MORE]);
        encode_into(&mut *self.sink, value)
    }

    fn field<T: Serialize + ?Sized>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> Result<(), CanonicalEncodeError> {
        self.sink.put(&[MORE]);
        put_text(self.sink, name);
        encode_into(&mut *self.sink, value)
    }

    fn finish(mut self) -> Result<(), CanonicalEncodeError> {
        if self.pending_key.is_some() {
            return Err(CanonicalEncodeError);
        }
        self.entries.sort();
        for (key, value) in &self.entries {
            self.sink.put(&[MORE]);
            self.sink.put(key);
            self.sink.put(value);
        }
        self.sink.put(&[END]);
        Ok(())
    }
}

macro_rules! element_compound {
    ($($Trait:ident::$method:ident),*) => {$(
        impl<S: Sink> ser::$Trait for Compound<'_, S> {
            type Ok = ();
            type Error = CanonicalEncodeError;

            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.element(value)
            }

            fn end(self) -> Result<(), Self::Error> {
                self.finish()
            }
        }
    )*};
}

element_compound!(
    SerializeSeq::serialize_element,
    SerializeTuple::serialize_element,
    SerializeTupleStruct::serialize_field,
    SerializeTupleVariant::serialize_field
);

macro_rules! field_compound {
    ($($Trait:ident),*) => {$(
        impl<S: Sink> ser::$Trait for Compound<'_, S> {
            type Ok = ();
            type Error = CanonicalEncodeError;

            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                name: &'static str,
                value: &T,
            ) -> Result<(), Self::Error> {
                self.field(name, value)
            }

            fn end(self) -> Result<(), Self::Error> {
                self.finish()
            }
        }
    )*};
}

field_compound!(SerializeStruct, SerializeStructVariant);

impl<S: Sink> ser::SerializeMap for Compound<'_, S> {
    type Ok = ();
    type Error = CanonicalEncodeError;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        if self.pending_key.is_some() {
            return Err(CanonicalEncodeError);
        }
        let key = encode_entry(key)?;
        self.sink
            .buffered(key.bytes.len().saturating_add(key.nested_buffered));
        self.pending_key = Some(key.bytes);
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.pending_key.take().ok_or(CanonicalEncodeError)?;
        let value = encode_entry(value)?;
        self.sink
            .buffered(value.bytes.len().saturating_add(value.nested_buffered));
        self.entries.push((key, value.bytes));
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
