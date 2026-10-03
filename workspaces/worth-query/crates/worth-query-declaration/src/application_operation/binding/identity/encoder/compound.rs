//! Compound values in progress: ordered members stream through, map entries wait.

use serde::ser::{self, Serialize};
use std::cmp::Ordering;

use super::{
    encode_entry, encode_into,
    serializer::{put_text, END, MORE},
    sink::{reserve, CanonicalEncodingCharge},
    CanonicalEncodeError, Sink,
};

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

    fn element<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        self.sink.put(&[MORE])?;
        encode_into(self.sink, value)
    }

    fn field<T: Serialize + ?Sized>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        self.sink.put(&[MORE])?;
        put_text(self.sink, name)?;
        encode_into(self.sink, value)
    }

    fn compare(
        &mut self,
        left: usize,
        right: usize,
    ) -> Result<Ordering, CanonicalEncodeError<S::AdmissionError>> {
        let (left_key, left_value) = &self.entries[left];
        let (right_key, right_value) = &self.entries[right];
        let key_visits = left_key
            .len()
            .min(right_key.len())
            .checked_add(1)
            .ok_or_else(|| self.sink.fail_capacity())?;
        let units = u64::try_from(key_visits).map_err(|_| self.sink.fail_capacity())?;
        self.sink.admit(CanonicalEncodingCharge::Work(units))?;
        let key_order = left_key.cmp(right_key);
        if key_order != Ordering::Equal {
            return Ok(key_order);
        }
        let value_visits = left_value
            .len()
            .min(right_value.len())
            .checked_add(1)
            .ok_or_else(|| self.sink.fail_capacity())?;
        let units = u64::try_from(value_visits).map_err(|_| self.sink.fail_capacity())?;
        self.sink.admit(CanonicalEncodingCharge::Work(units))?;
        Ok(left_value.cmp(right_value))
    }

    fn swap(
        &mut self,
        left: usize,
        right: usize,
    ) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        self.sink.admit(CanonicalEncodingCharge::Work(1))?;
        self.entries.swap(left, right);
        Ok(())
    }

    fn sift_down(
        &mut self,
        mut root: usize,
        end: usize,
    ) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        loop {
            let left = root
                .checked_mul(2)
                .and_then(|index| index.checked_add(1))
                .ok_or_else(|| self.sink.fail_capacity())?;
            if left >= end {
                break;
            }
            let mut child = left;
            if left + 1 < end && self.compare(left, left + 1)? == Ordering::Less {
                child = left + 1;
            }
            if self.compare(root, child)? != Ordering::Less {
                break;
            }
            self.swap(root, child)?;
            root = child;
        }
        Ok(())
    }

    fn sort_entries(&mut self) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        let count = self.entries.len();
        for root in (0..count / 2).rev() {
            self.sift_down(root, count)?;
        }
        for end in (1..count).rev() {
            self.swap(0, end)?;
            self.sift_down(0, end)?;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
        if self.pending_key.is_some() {
            return Err(CanonicalEncodeError::Serialization);
        }
        self.sort_entries()?;
        for (key, value) in &self.entries {
            self.sink.put(&[MORE])?;
            self.sink.put(key)?;
            self.sink.put(value)?;
        }
        self.sink.put(&[END])
    }
}

macro_rules! element_compound {
    ($($Trait:ident::$method:ident),*) => {$(
        impl<S: Sink> ser::$Trait for Compound<'_, S> {
            type Ok = ();
            type Error = CanonicalEncodeError<S::AdmissionError>;

            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.element(value)
            }

            fn end(self) -> Result<(), Self::Error> { self.finish() }
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
            type Error = CanonicalEncodeError<S::AdmissionError>;

            fn serialize_field<T: Serialize + ?Sized>(
                &mut self, name: &'static str, value: &T
            ) -> Result<(), Self::Error> { self.field(name, value) }

            fn end(self) -> Result<(), Self::Error> { self.finish() }
        }
    )*};
}

field_compound!(SerializeStruct, SerializeStructVariant);

impl<S: Sink> ser::SerializeMap for Compound<'_, S> {
    type Ok = ();
    type Error = CanonicalEncodeError<S::AdmissionError>;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        if self.pending_key.is_some() {
            return Err(CanonicalEncodeError::Serialization);
        }
        let (bytes, nested) = encode_entry(self.sink, key)?.finish();
        let buffered = bytes
            .len()
            .checked_add(nested)
            .ok_or_else(|| self.sink.fail_capacity())?;
        self.sink.buffered(buffered)?;
        self.pending_key = Some(bytes);
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        if self.pending_key.is_none() {
            return Err(CanonicalEncodeError::Serialization);
        }
        let (bytes, nested) = encode_entry(self.sink, value)?.finish();
        let buffered = bytes
            .len()
            .checked_add(nested)
            .ok_or_else(|| self.sink.fail_capacity())?;
        self.sink.buffered(buffered)?;
        reserve(self.sink, &mut self.entries, 1)?;
        let key = self.pending_key.take().expect("checked above");
        self.entries.push((key, bytes));
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
