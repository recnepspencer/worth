//! The existing named MessagePack grammar, in execution-owned byte storage.
use super::*;
use std::io::{self, Write};
use worth_execution::{ExecutionByteBuffer, ExecutionImmutableBytes, ExecutionRequest};
use RelationalAuthorizationDependencyDenial as Denial;
mod inventory;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationDependencyWire(ExecutionImmutableBytes);
impl RelationalAuthorizationDependencyWire {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.0.bytes()).expect("native hex is UTF8")
    }
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.0.charged_payload_bytes()
    }
}
impl std::ops::Deref for RelationalAuthorizationDependencyWire {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for RelationalAuthorizationDependencyWire {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl Serialize for RelationalAuthorizationDependencyWire {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl RelationalAuthorizationDurableDependencies {
    pub fn to_wire(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationDependencyWire, Denial> {
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        let bytes = self.wire_bytes(execution)?;
        let mut wire =
            ExecutionByteBuffer::allocate(bytes.len() * 2, execution.allocation_policy())
                .map_err(Denial::Allocation)?;
        let alphabet = b"0123456789abcdef";
        for &byte in bytes.bytes() {
            check_live(execution)?;
            wire.extend_from_slice(&[
                alphabet[(byte >> 4) as usize],
                alphabet[(byte & 15) as usize],
            ])
            .map_err(Denial::Allocation)?;
        }
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        Ok(RelationalAuthorizationDependencyWire(
            wire.seal().map_err(Denial::Allocation)?,
        ))
    }
    /// Descriptive evidence only. The deep parse payer follows every stamp clone.
    pub fn from_wire(execution: &ExecutionRequest<'_, '_>, wire: &str) -> Result<Self, Denial> {
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        check_live(execution)?;
        if wire.len() > MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES * 2 {
            return Err(Denial::ByteBudgetExceeded);
        }
        if !wire.len().is_multiple_of(2) || wire.is_empty() {
            return Err(Denial::MalformedWire);
        }
        for pair in wire.as_bytes().chunks_exact(2) {
            check_live(execution)?;
            nibble(pair[0]).ok_or(Denial::MalformedWire)?;
            nibble(pair[1]).ok_or(Denial::MalformedWire)?;
        }
        let mut bytes =
            ExecutionByteBuffer::allocate(wire.len() / 2, execution.allocation_policy())
                .map_err(Denial::Allocation)?;
        for pair in wire.as_bytes().chunks_exact(2) {
            check_live(execution)?;
            bytes
                .extend_from_slice(&[(nibble(pair[0]).ok_or(Denial::MalformedWire)? << 4)
                    | nibble(pair[1]).ok_or(Denial::MalformedWire)?])
                .map_err(Denial::Allocation)?;
        }
        // SAME serde vocabulary, first as allocation inventory with borrowed strings.
        let inventory: inventory::Inventory =
            rmp_serde::from_slice(bytes.bytes()).map_err(|_| Denial::MalformedWire)?;
        if inventory.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        let storage = inventory.quote()?.admit(execution)?;
        let data: DependencyData =
            rmp_serde::from_slice(bytes.bytes()).map_err(|_| Denial::MalformedWire)?;
        let decoded = Self {
            owned: Arc::new(OwnedDependencies {
                data,
                _storage: storage,
            }),
        };
        let canonical = decoded.wire_bytes(execution)?;
        if canonical.bytes() != bytes.bytes() {
            return Err(Denial::MalformedWire);
        }
        check_live(execution)?;
        execution
            .consult()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        Ok(decoded)
    }
    pub(super) fn check_wire_limits(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<usize, Denial> {
        if self.owned.data.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        if self.dependency_count() > MAXIMUM_AUTHORIZATION_DEPENDENCIES {
            return Err(Denial::DependencyBudgetExceeded);
        }
        let mut writer = WireWriter {
            execution,
            output: None,
            count: 0,
            denial: None,
        };
        let answer = rmp_serde::encode::write_named(&mut writer, &self.owned.data);
        writer.finish(answer)?;
        Ok(writer.count)
    }
    fn wire_bytes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, Denial> {
        let count = self.check_wire_limits(execution)?;
        let output = ExecutionByteBuffer::allocate(count, execution.allocation_policy())
            .map_err(Denial::Allocation)?;
        let mut writer = WireWriter {
            execution,
            output: Some(output),
            count: 0,
            denial: None,
        };
        let answer = rmp_serde::encode::write_named(&mut writer, &self.owned.data);
        writer.finish(answer)?;
        writer
            .output
            .take()
            .expect("serialization owns buffer")
            .seal()
            .map_err(Denial::Allocation)
    }
    fn dependency_count(&self) -> usize {
        self.owned.data.paths.iter().fold(2usize, |count, path| {
            count
                .saturating_add(1)
                .saturating_add(path.witness.as_ref().map_or(0, Vec::len))
                .saturating_add(path.entities.len())
                .saturating_add(path.relations.len())
                .saturating_add(path.adjacencies.len())
                .saturating_add(path.fields.len())
        })
    }
}

struct WireWriter<'a, 'scope, 'authority> {
    execution: &'a ExecutionRequest<'scope, 'authority>,
    output: Option<ExecutionByteBuffer>,
    count: usize,
    denial: Option<Denial>,
}
impl WireWriter<'_, '_, '_> {
    fn finish<T>(&mut self, result: Result<T, rmp_serde::encode::Error>) -> Result<(), Denial> {
        if let Some(denial) = self.denial.take() {
            return Err(denial);
        }
        result.map(|_| ()).map_err(|_| Denial::MalformedWire)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), Denial> {
        check_live(self.execution)?;
        let count = self
            .count
            .checked_add(bytes.len())
            .filter(|count| *count <= MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES)
            .ok_or(Denial::ByteBudgetExceeded)?;
        if let Some(output) = &mut self.output {
            output
                .extend_from_slice(bytes)
                .map_err(Denial::Allocation)?;
        }
        self.count = count;
        Ok(())
    }
}
impl Write for WireWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.append(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(denial) => {
                self.denial = Some(denial);
                Err(io::Error::other("authorization wire stopped"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn check_live(execution: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    execution
        .check_live()
        .map_err(|stop| Denial::Execution(stop.into()))
}
fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
