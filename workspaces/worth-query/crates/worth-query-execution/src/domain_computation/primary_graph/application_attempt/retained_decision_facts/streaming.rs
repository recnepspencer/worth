use super::{AdmittedFactKey, StoreDenial};
use std::fmt;
use worth_execution::ExecutionByteBuffer;
use worth_foundational::facade::{
    write_aspect_value_identity_material, AspectValue, CanonicalMaterialSink,
};

impl AdmittedFactKey {
    /// The caller writes the existing locator grammar directly from borrowed
    /// identity fields. Predicate grammar belongs to Foundational, not this sink.
    pub(in crate::domain_computation::primary_graph) fn from_borrowed(
        locator: impl Fn(&mut dyn fmt::Write) -> fmt::Result,
        predicate: Option<(&AspectValue, usize)>,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let mut locator_count = Sink::count(policy);
        locator_count.format(&locator)?;
        let predicate_count = if let Some((value, limit)) = predicate {
            let mut count = Sink::count(policy);
            write_aspect_value_identity_material(value, &mut count)?;
            Some((count.length, limit))
        } else {
            None
        };
        Self::write(
            locator_count.length,
            predicate_count,
            policy,
            |buffer| Sink::bytes(buffer, policy).format(&locator),
            |buffer| {
                write_aspect_value_identity_material(
                    predicate.expect("predicate writer runs only for Some").0,
                    &mut Sink::bytes(buffer, policy),
                )
            },
        )
    }
}

enum Destination<'buffer> {
    Count,
    Bytes(&'buffer mut ExecutionByteBuffer),
}
struct Sink<'buffer, 'scope, 'authority> {
    destination: Destination<'buffer>,
    policy: super::StorageControl<'scope, 'authority>,
    length: usize,
    format_denial: Option<StoreDenial>,
}
impl<'buffer, 'scope, 'authority> Sink<'buffer, 'scope, 'authority> {
    fn count(policy: super::StorageControl<'scope, 'authority>) -> Self {
        Self {
            destination: Destination::Count,
            policy,
            length: 0,
            format_denial: None,
        }
    }
    fn bytes(
        buffer: &'buffer mut ExecutionByteBuffer,
        policy: super::StorageControl<'scope, 'authority>,
    ) -> Self {
        Self {
            destination: Destination::Bytes(buffer),
            policy,
            length: 0,
            format_denial: None,
        }
    }
    fn format(
        &mut self,
        writer: &impl Fn(&mut dyn fmt::Write) -> fmt::Result,
    ) -> Result<(), StoreDenial> {
        match writer(self) {
            Ok(()) => {
                self.policy.check_live()?;
                Ok(())
            }
            Err(_) => Err(self
                .format_denial
                .take()
                .unwrap_or(StoreDenial::Representability)),
        }
    }
}
impl CanonicalMaterialSink for Sink<'_, '_, '_> {
    type Error = StoreDenial;
    fn append(&mut self, value: &str) -> Result<(), StoreDenial> {
        if let Destination::Bytes(buffer) = &self.destination {
            buffer.check_live()?;
        }
        self.policy.check_live()?;
        let length = self
            .length
            .checked_add(value.len())
            .ok_or(StoreDenial::Representability)?;
        if let Destination::Bytes(buffer) = &mut self.destination {
            for chunk in value.as_bytes().chunks(64 * 1024) {
                self.policy.check_live()?;
                buffer.extend_from_slice(chunk)?;
            }
        }
        self.length = length;
        self.policy.check_live()?;
        Ok(())
    }
    fn admit_work(&mut self, _: usize) -> Result<(), StoreDenial> {
        self.policy.check_live()?;
        Ok(())
    }
    fn accounting_overflow(&mut self) -> StoreDenial {
        StoreDenial::Representability
    }
}
impl fmt::Write for Sink<'_, '_, '_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        CanonicalMaterialSink::append(self, value).map_err(|denial| {
            self.format_denial = Some(denial);
            fmt::Error
        })
    }
}
