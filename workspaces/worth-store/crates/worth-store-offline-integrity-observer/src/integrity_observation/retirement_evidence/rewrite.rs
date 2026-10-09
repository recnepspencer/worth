use super::super::{
    families::durable_frame::{read_u32, read_u64},
    sha256::sha256,
};

#[derive(Clone)]
pub(super) struct RewriteFact {
    pub(super) source_root: u64,
    pub(super) result_root: u64,
    pub(super) extent: u64,
    pub(super) source_generation: u64,
    pub(super) destination_generation: u64,
    pub(super) source: (u64, u64, u64),
    pub(super) destination: (u64, u64, u64),
    pub(super) record: [u8; 24],
    pub(super) logical_bytes: u64,
}

impl RewriteFact {
    /// Parse the two length-delimited member fields, then the exact rewrite
    /// domain. Never search user record payloads for a domain-shaped substring.
    pub(super) fn decode(payload: &[u8], store: [u8; 16], lsn: (u64, u64)) -> Option<Self> {
        let mut outer = Cursor(payload);
        let binding = outer.field()?;
        let redo = outer.field()?;
        if !outer.0.is_empty() {
            return None;
        }
        let mut body = Cursor(redo);
        if body.field()? != b"store.physical.rewrite-redo.v2" || body.0.len() != 280 {
            return None;
        }
        let bytes = body.0;
        let mut binding = Cursor(binding);
        if binding.field()? != b"store.physical.mutation-attempt-binding.v1" {
            return None;
        }
        binding.fixed_field(32)?;
        if binding.fixed_field(16)? != store {
            return None;
        }
        binding.fixed_field(32)?;
        binding.take(16)?;
        binding.fixed_field(32)?;
        binding.fixed_field(32)?;
        if binding.fixed_field(16)? != store {
            return None;
        }
        binding.take(24)?;
        let group = binding.fixed_field(32)?;
        let member = binding.take(8)?;
        if read_u32(member, 0) == 0 || read_u32(member, 0) > read_u32(member, 4) {
            return None;
        }
        binding.fixed_field(32)?;
        binding.fixed_field(32)?;
        let interval = binding.take(16)?;
        if (read_u64(interval, 0), read_u64(interval, 8)) != lsn
            || binding.fixed_field(32)? != sha256(redo)
            || !binding.0.is_empty()
            || bytes[32..64] != *group
            || read_u64(bytes, 144) != lsn.0
        {
            return None;
        }
        let source_root = read_u64(bytes, 64);
        let source_generation = read_u64(bytes, 72);
        let destination_generation = read_u64(bytes, 124);
        let extent = read_u64(bytes, 184);
        let result_root = read_u64(bytes, 208);
        let source = (
            read_u64(bytes, 224),
            read_u64(bytes, 232),
            read_u64(bytes, 240),
        );
        let destination = (
            read_u64(bytes, 248),
            read_u64(bytes, 256),
            read_u64(bytes, 264),
        );
        let alignment = read_u64(bytes, 272);
        if source_root == 0
            || source_root.checked_add(1) != Some(result_root)
            || source_generation == 0
            || destination_generation != source_generation.checked_add(1)?
            || extent == 0
            || read_u64(bytes, 192) != extent
            || read_u64(bytes, 216) != 1
            || read_u32(bytes, 88) == 0
            || read_u32(bytes, 88) != read_u32(bytes, 140)
            || read_u64(bytes, 200) != u64::from(read_u32(bytes, 140))
            || bytes[176..184] != [0; 8]
            || bytes[152..168] == [0; 16]
            || read_u64(bytes, 168) == 0
            || !alignment.is_power_of_two()
            || source.0 == 0
            || destination.0 == 0
            || source.2 == 0
            || source.2 != destination.2
            || source.2 % alignment != 0
            || source.1 % alignment != 0
            || destination.1 % alignment != 0
            || source.1.checked_add(source.2).is_none()
            || destination.1.checked_add(destination.2).is_none()
            || (source.0 == destination.0
                && source.1 < destination.1 + destination.2
                && destination.1 < source.1 + source.2)
        {
            return None;
        }
        Some(Self {
            source_root,
            result_root,
            extent,
            source_generation,
            destination_generation,
            source,
            destination,
            record: bytes[152..176].try_into().ok()?,
            logical_bytes: u64::from(read_u32(bytes, 88)),
        })
    }
}

struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let result = self.0.get(..length)?;
        self.0 = &self.0[length..];
        Some(result)
    }
    fn field(&mut self) -> Option<&'a [u8]> {
        let length = usize::try_from(read_u64(self.take(8)?, 0)).ok()?;
        self.take(length)
    }
    fn fixed_field(&mut self, length: usize) -> Option<&'a [u8]> {
        self.field().filter(|field| field.len() == length)
    }
}
