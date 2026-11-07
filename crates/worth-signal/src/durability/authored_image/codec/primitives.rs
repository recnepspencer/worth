use super::Denial;
use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::handle::NodeId;
use crate::data::output::{PartitionSubscription, ScopeCoverage, ScopePath};

pub(crate) struct Write<'a> {
    emit: &'a mut dyn FnMut(&[u8]) -> Result<(), Denial>,
}
impl<'a> Write<'a> {
    pub(crate) fn new(emit: &'a mut dyn FnMut(&[u8]) -> Result<(), Denial>) -> Self {
        Self { emit }
    }
    pub(crate) fn byte(&mut self, value: u8) -> Result<(), Denial> {
        (self.emit)(&[value])
    }
    pub(crate) fn boolean(&mut self, value: bool) -> Result<(), Denial> {
        self.byte(u8::from(value))
    }
    pub(crate) fn number(&mut self, value: u64) -> Result<(), Denial> {
        (self.emit)(&value.to_le_bytes())
    }
    pub(crate) fn small_number(&mut self, value: u32) -> Result<(), Denial> {
        (self.emit)(&value.to_le_bytes())
    }
    pub(crate) fn count(&mut self, value: usize) -> Result<(), Denial> {
        self.small_number(value.try_into().map_err(|_| Denial::SizeLimit)?)
    }
    pub(super) fn text(&mut self, value: &str) -> Result<(), Denial> {
        self.count(value.len())?;
        (self.emit)(value.as_bytes())
    }
    pub(super) fn optional_text(&mut self, value: Option<&str>) -> Result<(), Denial> {
        self.boolean(value.is_some())?;
        if let Some(value) = value {
            self.text(value)?;
        }
        Ok(())
    }
    pub(crate) fn node(&mut self, node: NodeId) -> Result<(), Denial> {
        self.small_number(node.index())?;
        self.small_number(node.generation())
    }
    pub(super) fn mask(&mut self, value: AspectMask) -> Result<(), Denial> {
        (self.emit)(&(value.bits() as u128).to_le_bytes())
    }
    pub(super) fn scope(&mut self, scope: Option<&PartitionSubscription>) -> Result<(), Denial> {
        self.boolean(scope.is_some())?;
        if let Some(scope) = scope {
            self.byte(match scope.coverage() {
                ScopeCoverage::Exact => 0,
                ScopeCoverage::Subtree => 1,
            })?;
            self.count(scope.path().depth())?;
            for segment in scope.path().segments() {
                self.text(segment)?;
            }
        }
        Ok(())
    }
    pub(super) fn scopes(
        &mut self,
        scopes: Option<&[PartitionSubscription]>,
    ) -> Result<(), Denial> {
        self.boolean(scopes.is_some())?;
        if let Some(scopes) = scopes {
            self.count(scopes.len())?;
            for scope in scopes {
                self.scope(Some(scope))?;
            }
        }
        Ok(())
    }
}

pub(super) struct Read<'a> {
    bytes: &'a [u8],
    /// Validation scans allocate no variable-size declaration payloads.
    pub(super) materialize: bool,
    pub(super) graph_instance: u64,
    pub(super) node: NodeId,
}
impl<'a> Read<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            materialize: false,
            graph_instance: 0,
            node: NodeId::new(0, 0),
        }
    }
    pub(super) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Denial> {
        let (value, rest) = self
            .bytes
            .split_first_chunk::<N>()
            .ok_or(Denial::Malformed)?;
        self.bytes = rest;
        Ok(*value)
    }
    pub(super) fn byte(&mut self) -> Result<u8, Denial> {
        Ok(self.fixed::<1>()?[0])
    }
    pub(super) fn boolean(&mut self) -> Result<bool, Denial> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Denial::Malformed),
        }
    }
    pub(super) fn number(&mut self) -> Result<u64, Denial> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }
    pub(super) fn small_number(&mut self) -> Result<u32, Denial> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }
    pub(super) fn count(&mut self) -> Result<usize, Denial> {
        let count = self.small_number()? as usize;
        if count > self.bytes.len() {
            return Err(Denial::Malformed);
        }
        Ok(count)
    }
    pub(super) fn text(&mut self) -> Result<String, Denial> {
        let length = self.count()?;
        let (text, rest) = self
            .bytes
            .split_at_checked(length)
            .ok_or(Denial::Malformed)?;
        let text = std::str::from_utf8(text).map_err(|_| Denial::Malformed)?;
        self.bytes = rest;
        Ok(if self.materialize {
            text.to_owned()
        } else {
            String::new()
        })
    }
    pub(super) fn optional_text(&mut self) -> Result<Option<String>, Denial> {
        if self.boolean()? {
            Ok(Some(self.text()?))
        } else {
            Ok(None)
        }
    }
    pub(super) fn node_id(&mut self) -> Result<NodeId, Denial> {
        Ok(NodeId::new(self.small_number()?, self.small_number()?))
    }
    pub(super) fn aspect(&mut self) -> Result<Aspect, Denial> {
        let index = self.byte()?;
        if index as usize >= MAX_ASPECTS {
            return Err(Denial::Malformed);
        }
        Ok(Aspect::new(index))
    }
    pub(super) fn mask(&mut self) -> Result<AspectMask, Denial> {
        let value = u128::from_le_bytes(self.fixed()?);
        if value & !(AspectMask::ALL.bits() as u128) != 0 {
            return Err(Denial::Malformed);
        }
        Ok(AspectMask::from_bits(value as _))
    }
    pub(super) fn list<T>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> Result<T, Denial>,
    ) -> Result<Vec<T>, Denial> {
        let count = self.count()?;
        let mut values = if self.materialize {
            Vec::with_capacity(count)
        } else {
            Vec::new()
        };
        for _ in 0..count {
            let value = read(self)?;
            if self.materialize {
                values.push(value);
            }
        }
        Ok(values)
    }
    pub(super) fn scope(&mut self) -> Result<Option<PartitionSubscription>, Denial> {
        if !self.boolean()? {
            return Ok(None);
        }
        let coverage = match self.byte()? {
            0 => ScopeCoverage::Exact,
            1 => ScopeCoverage::Subtree,
            _ => return Err(Denial::Malformed),
        };
        let count = self.count()?;
        if count == 0 || count > ScopePath::MAX_DEPTH {
            return Err(Denial::Malformed);
        }
        let mut segments = Vec::new();
        for _ in 0..count {
            // Validate actual UTF-8/length before replacing text during the scan.
            let length = self.small_number()? as usize;
            let (text, rest) = self
                .bytes
                .split_at_checked(length)
                .ok_or(Denial::Malformed)?;
            if text.is_empty() || std::str::from_utf8(text).is_err() {
                return Err(Denial::Malformed);
            }
            self.bytes = rest;
            if self.materialize {
                segments.push(std::str::from_utf8(text).unwrap().to_owned());
            }
        }
        if !self.materialize {
            return Ok(None);
        }
        let path = ScopePath::new(segments).map_err(|_| Denial::Malformed)?;
        Ok(Some(match coverage {
            ScopeCoverage::Exact => PartitionSubscription::exact(path),
            ScopeCoverage::Subtree => PartitionSubscription::subtree(path),
        }))
    }
    pub(super) fn scopes(&mut self) -> Result<Option<Vec<PartitionSubscription>>, Denial> {
        if !self.boolean()? {
            return Ok(None);
        }
        Ok(Some(
            self.list(|read| {
                if read.bytes.first() != Some(&1) {
                    return Err(Denial::Malformed);
                }
                read.scope()
            })?
            .into_iter()
            .flatten()
            .collect(),
        ))
    }
    pub(super) fn finish(self) -> Result<(), Denial> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(Denial::Malformed)
        }
    }
}
