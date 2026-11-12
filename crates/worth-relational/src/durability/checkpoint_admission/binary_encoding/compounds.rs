//! Preserve every composite's structure while applying the byte policy to children.
use super::BinaryFixedArrays;
use serde::{ser::*, Serialize};
pub(super) struct Compound<C>(pub(super) C);
impl<C: SerializeSeq> SerializeSeq for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.serialize_element(&BinaryFixedArrays(value))
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
impl<C: SerializeTupleStruct> SerializeTupleStruct for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.serialize_field(&BinaryFixedArrays(value))
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
impl<C: SerializeTupleVariant> SerializeTupleVariant for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.serialize_field(&BinaryFixedArrays(value))
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
impl<C: SerializeMap> SerializeMap for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.serialize_key(&BinaryFixedArrays(value))
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.serialize_value(&BinaryFixedArrays(value))
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
impl<C: SerializeStruct> SerializeStruct for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.0.serialize_field(key, &BinaryFixedArrays(value))
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.0.skip_field(key)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
impl<C: SerializeStructVariant> SerializeStructVariant for Compound<C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.0.serialize_field(key, &BinaryFixedArrays(value))
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.0.skip_field(key)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}
