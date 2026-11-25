//! Field-wise remote wire codec: bounded field encoding through
//! [`RemoteByteWriter`], strict decoding through [`RemoteByteReader`], and
//! [`crate::worth_query_remote_codec!`] composing them into a
//! [`RemoteBinaryCodec`] for query payloads and rows.

use super::binding::RemoteBinaryCodec;
use super::capped_writer::RemoteByteWriter;
use super::denial::{ProtocolRefusalReason, SharedAccessDenial};

fn malformed() -> SharedAccessDenial {
    SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::MalformedPayload)
}

/// Strict cursor over one remote payload; every read is bounds-checked and
/// [`Self::finish`] refuses trailing bytes.
pub struct RemoteByteReader<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl<'bytes> RemoteByteReader<'bytes> {
    pub const fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'bytes [u8], SharedAccessDenial> {
        let end = self.position.checked_add(count).ok_or_else(malformed)?;
        let taken = self.bytes.get(self.position..end).ok_or_else(malformed)?;
        self.position = end;
        Ok(taken)
    }

    pub fn read_u8(&mut self) -> Result<u8, SharedAccessDenial> {
        Ok(self.take(1)?[0])
    }

    pub fn read_u32(&mut self) -> Result<u32, SharedAccessDenial> {
        let bytes = self.take(4)?.try_into().map_err(|_| malformed())?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_u64(&mut self) -> Result<u64, SharedAccessDenial> {
        let bytes = self.take(8)?.try_into().map_err(|_| malformed())?;
        Ok(u64::from_be_bytes(bytes))
    }

    /// Reads one `u32`-length-prefixed byte string.
    pub fn read_bytes(&mut self) -> Result<&'bytes [u8], SharedAccessDenial> {
        let length = usize::try_from(self.read_u32()?).map_err(|_| malformed())?;
        self.take(length)
    }

    pub fn read_str(&mut self) -> Result<&'bytes str, SharedAccessDenial> {
        std::str::from_utf8(self.read_bytes()?).map_err(|_| malformed())
    }

    /// Completes decoding, refusing any unread bytes.
    pub fn finish(self) -> Result<(), SharedAccessDenial> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::TrailingBytes,
            ))
        }
    }
}

/// One field of a remote payload or row.
pub trait RemoteBinaryField: Sized {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial>;
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial>;
}

impl RemoteBinaryField for String {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.write_str(self)
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        reader.read_str().map(str::to_owned)
    }
}

impl RemoteBinaryField for u64 {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.write_u64(*self)
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        reader.read_u64()
    }
}

impl RemoteBinaryField for i64 {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.write_fixed(&self.to_be_bytes())
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        let bytes = reader.take(8)?.try_into().map_err(|_| malformed())?;
        Ok(i64::from_be_bytes(bytes))
    }
}

impl RemoteBinaryField for u32 {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.write_u32(*self)
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        reader.read_u32()
    }
}

impl RemoteBinaryField for bool {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        writer.write_u8(u8::from(*self))
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        match reader.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(malformed()),
        }
    }
}

impl<T: RemoteBinaryField> RemoteBinaryField for Option<T> {
    fn encode_field(&self, writer: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        match self {
            None => writer.write_u8(0),
            Some(value) => {
                writer.write_u8(1)?;
                value.encode_field(writer)
            }
        }
    }
    fn decode_field(reader: &mut RemoteByteReader<'_>) -> Result<Self, SharedAccessDenial> {
        match reader.read_u8()? {
            0 => Ok(None),
            1 => T::decode_field(reader).map(Some),
            _ => Err(malformed()),
        }
    }
}

/// The empty query payload.
impl RemoteBinaryCodec for () {
    fn encode_remote_bounded(&self, _: &mut RemoteByteWriter) -> Result<(), SharedAccessDenial> {
        Ok(())
    }
    fn decode_remote(bytes: &[u8]) -> Result<Self, SharedAccessDenial> {
        RemoteByteReader::new(bytes).finish()
    }
}

/// Implements [`RemoteBinaryCodec`] for a struct from its fields, in order.
///
/// `worth_query_remote_codec!(ProjectRecord { id, name });` encodes each
/// [`RemoteBinaryField`] through the bounded writer and decodes them strictly,
/// refusing malformed or trailing bytes. `worth_query_remote_codec!(Params);`
/// covers a field-less unit struct.
#[macro_export]
macro_rules! worth_query_remote_codec {
    ($Type:ident) => {
        impl $crate::facade::shared_access::RemoteBinaryCodec for $Type {
            fn encode_remote_bounded(
                &self,
                _: &mut $crate::facade::shared_access::RemoteByteWriter,
            ) -> ::core::result::Result<(), $crate::facade::shared_access::SharedAccessDenial> {
                ::core::result::Result::Ok(())
            }
            fn decode_remote(
                bytes: &[u8],
            ) -> ::core::result::Result<Self, $crate::facade::shared_access::SharedAccessDenial> {
                $crate::facade::shared_access::RemoteByteReader::new(bytes).finish()?;
                ::core::result::Result::Ok($Type)
            }
        }
    };
    ($Type:ident { $($field:ident),+ $(,)? }) => {
        impl $crate::facade::shared_access::RemoteBinaryCodec for $Type {
            fn encode_remote_bounded(
                &self,
                writer: &mut $crate::facade::shared_access::RemoteByteWriter,
            ) -> ::core::result::Result<(), $crate::facade::shared_access::SharedAccessDenial> {
                $($crate::facade::shared_access::RemoteBinaryField::encode_field(&self.$field, writer)?;)+
                ::core::result::Result::Ok(())
            }
            fn decode_remote(
                bytes: &[u8],
            ) -> ::core::result::Result<Self, $crate::facade::shared_access::SharedAccessDenial> {
                let mut reader = $crate::facade::shared_access::RemoteByteReader::new(bytes);
                $(let $field = $crate::facade::shared_access::RemoteBinaryField::decode_field(&mut reader)?;)+
                reader.finish()?;
                ::core::result::Result::Ok($Type { $($field,)+ })
            }
        }
    };
}
