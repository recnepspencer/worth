use std::path::Path;

const MAGIC: &[u8] = b"WORTH-C8-SUBMITTED-OPERATIONS-V1\n";
const MATERIAL_BYTES: usize = 32;

pub(super) struct C8OperationProgram {
    operations: Vec<C8Operation>,
}

pub(super) struct C8Operation {
    payload: Vec<u8>,
    material: [u8; MATERIAL_BYTES],
}

impl C8OperationProgram {
    pub(super) fn scheduled_operations(&self, seed: u64) -> Vec<&C8Operation> {
        let mut order = (0..self.operations.len()).collect::<Vec<_>>();
        let mut state = seed ^ 0xC8_5C_4A_01;
        for index in (1..order.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let swap = (state as usize) % (index + 1);
            order.swap(index, swap);
        }
        order
            .into_iter()
            .map(|index| &self.operations[index])
            .collect()
    }
}

impl C8Operation {
    pub(super) fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub(super) const fn material(&self) -> [u8; MATERIAL_BYTES] {
        self.material
    }
}

pub(super) fn read(path: &Path) -> Result<C8OperationProgram, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("read C8 submitted operation program {path:?}: {error}"))?;
    let mut cursor = Cursor::new(&bytes);
    if cursor.take(MAGIC.len())? != MAGIC {
        return Err("C8 submitted operation program has an invalid magic".to_owned());
    }
    let operation_count = cursor.take_u32()? as usize;
    if operation_count == 0 {
        return Err("C8 submitted operation program has no operations".to_owned());
    }
    let mut operations = Vec::with_capacity(operation_count);
    for _ in 0..operation_count {
        let material = cursor.take_array::<MATERIAL_BYTES>()?;
        let payload_length = cursor.take_u64()? as usize;
        if payload_length == 0 {
            return Err("C8 submitted operation program has an empty payload".to_owned());
        }
        operations.push(C8Operation {
            payload: cursor.take(payload_length)?.to_owned(),
            material,
        });
    }
    if !cursor.is_empty() {
        return Err("C8 submitted operation program has trailing bytes".to_owned());
    }
    Ok(C8OperationProgram { operations })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| "C8 submitted operation program length overflow".to_owned())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "C8 submitted operation program is truncated".to_owned())?;
        self.offset = end;
        Ok(value)
    }

    fn take_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take_array()?))
    }

    fn take_u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take_array()?))
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N).and_then(|value| {
            value
                .try_into()
                .map_err(|_| "C8 submitted operation program array width mismatch".to_owned())
        })
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
