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
        writer.bytes(if self.comparison_depth > 2 {
            b"worth:projected-source-entry:v2"
        } else {
            b"worth:projected-source-entry:v1"
        })?;
        writer.entity(self.occurrence.entity())?;
        if self.comparison_depth > 2 {
            self.row.append_tree_content(&mut writer, self.comparison_depth,
                &mut |bytes| control.reserve_content_memory(bytes),
                &mut |cause| control.refuse(match cause {
                    crate::domain_computation::primary_graph::application_query::ProjectionContentStorageFault::CapacityOverflow => Stop::CapacityOverflow,
                    cause => Stop::MaterialStorage(cause.into_native()),
                }))?;
        } else {
            self.row
                .append_one_hop_content(&mut writer, || control.refuse(Stop::IncompleteCoverage))?;
        }
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

impl Entry {
    /// The caller supplies the original paid comparison checkpoint. A cached
    /// commitment can reject a candidate, but can never establish equality.
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn same_content<
        E,
    >(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        reserve: &mut (impl FnMut(u64) -> Result<Option<worth_execution::ExecutionMemoryReservation>, E>
                  + ?Sized),
        fault: &mut impl FnMut(crate::domain_computation::primary_graph::application_query::ProjectionContentComparisonFault) -> E,
    ) -> Result<bool, E> {
        admit(65)?;
        if self.occurrence != other.occurrence
            || self.dependencies.len() != other.dependencies.len()
            || matches!((self.content.get(), other.content.get()), (Some(a), Some(b)) if a != b)
        {
            return Ok(false);
        }
        for (left, right) in self.dependencies.iter().zip(&other.dependencies) {
            admit_dependency(left, admit)?;
            admit_dependency(right, admit)?;
            if left != right {
                return Ok(false);
            }
        }
        self.row.same_source_content(
            &other.row,
            self.comparison_depth,
            other.comparison_depth,
            admit,
            reserve,
            fault,
        )
    }
}

fn admit_dependency<E>(
    dependency: &ViewDependency,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<(), E> {
    admit(32)?;
    match dependency {
        ViewDependency::Entity(_) | ViewDependency::Adjacency(..) => {}
        ViewDependency::Aspect(_, aspect) => admit(aspect.as_str().len() as u64)?,
        ViewDependency::Field(_, locator) => {
            admit(locator.aspect().aspect_key().as_str().len() as u64)?;
            for field in locator.field_path().fields() {
                admit(1)?;
                admit(field.as_str().len() as u64)?;
            }
        }
    }
    Ok(())
}
