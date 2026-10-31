//! Lazy complete Entry content; the immutable Entry pays its inline cache.
use super::projection::Entry;
use crate::domain_computation::primary_graph::application_query::{
    derived_view::ViewDependency,
    managed_index::{WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexStop as Stop},
    projection::ContentWriter,
};
impl Entry {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn content(
        &self,
        control: &Control<'_>,
    ) -> Result<[u8; 32], Stop> {
        control.checkpoint(1)?;
        if let Some(value) = self.content.get() {
            return Ok(*value);
        }
        let mut admit = |work| control.checkpoint(work);
        let overflow = || control.refuse(Stop::CapacityOverflow);
        let mut writer = ContentWriter::new(&mut admit, &overflow);
        writer.bytes(b"worth:projected-source-entry:v1")?;
        writer.entity(self.occurrence.entity())?;
        self.row
            .append_one_hop_content(&mut writer, || control.refuse(Stop::IncompleteCoverage))?;
        writer.number(self.dependencies.len() as u64)?;
        for dependency in &self.dependencies {
            writer.entity(dependency.entity())?;
            match dependency {
                ViewDependency::Entity(_) => writer.number(0)?,
                ViewDependency::Aspect(_, aspect) => {
                    writer.number(1)?;
                    writer.bytes(aspect.as_str().as_bytes())?;
                }
                ViewDependency::Field(_, locator) => {
                    writer.number(2)?;
                    writer.bytes(locator.aspect().aspect_key().as_str().as_bytes())?;
                    writer.number(locator.field_path().fields().len() as u64)?;
                    for field in locator.field_path().fields() {
                        writer.bytes(field.as_str().as_bytes())?;
                    }
                }
                ViewDependency::Adjacency(_, kind, direction) => {
                    writer.number(3)?;
                    writer.number(u64::from(kind.as_u32()))?;
                    writer.number(u64::from(*direction))?;
                }
            }
        }
        let value = writer.finish();
        control.poll()?;
        let _ = self.content.set(value);
        Ok(value)
    }
}
