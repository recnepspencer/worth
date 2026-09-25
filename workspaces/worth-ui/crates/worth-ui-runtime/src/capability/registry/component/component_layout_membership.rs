use std::collections::{BTreeMap, BTreeSet};

use crate::capability::{
    CapabilityDiagnosticCode, ComponentAllocationMeasurementContract, ComponentId,
    RegistrationCandidateDiagnostic,
};

use super::{ComponentDescriptor, ComponentRegistry};

impl ComponentRegistry {
    /// Layout facts no single descriptor can check: every layout-cell
    /// component belongs to exactly one container and is placed by nothing
    /// else, every member is a registered component that places itself
    /// within its cell, every container declares its own allocation, and no
    /// container is laid out inside itself.
    pub(crate) fn layout_membership_diagnostics(
        &self,
    ) -> Vec<(ComponentId, RegistrationCandidateDiagnostic)> {
        layout_membership_diagnostics(self.descriptors())
    }
}

fn layout_membership_diagnostics(
    descriptors: &[ComponentDescriptor],
) -> Vec<(ComponentId, RegistrationCandidateDiagnostic)> {
    let containers = containers_by_member(descriptors);
    let mut diagnostics = Vec::new();
    for descriptor in descriptors {
        let memberships = containers.get(descriptor.id()).map_or(0, Vec::len);
        if is_layout_cell(descriptor) && memberships != 1 {
            diagnostics.push((
                descriptor.id().clone(),
                RegistrationCandidateDiagnostic::new(
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership,
                    "a layout-cell component must belong to exactly one layout container",
                ),
            ));
        }
        if is_layout_cell(descriptor) && descriptor.portal_child_contract().is_some() {
            diagnostics.push((
                descriptor.id().clone(),
                RegistrationCandidateDiagnostic::new(
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership,
                    "a layout-cell component is placed by its container, not a Portal owner",
                ),
            ));
        }
        if descriptor.layout().is_some() && descriptor.allocation_measurement_contract().is_none() {
            diagnostics.push((
                descriptor.id().clone(),
                RegistrationCandidateDiagnostic::new(
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership,
                    "a layout container must declare the allocation its tracks divide",
                ),
            ));
        }
        let members_are_cells = descriptor.layout().is_none_or(|layout| {
            layout.fallback().members().all(|(member, _)| {
                descriptors
                    .iter()
                    .find(|candidate| candidate.id() == member)
                    .is_some_and(is_layout_cell)
            })
        });
        if !members_are_cells {
            diagnostics.push((
                descriptor.id().clone(),
                RegistrationCandidateDiagnostic::new(
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership,
                    "layout members must be registered components placed by a layout-cell allocation contract",
                ),
            ));
        }
        if lays_out_itself(descriptor.id(), &containers) {
            diagnostics.push((
                descriptor.id().clone(),
                RegistrationCandidateDiagnostic::new(
                    CapabilityDiagnosticCode::CyclicComponentLayout,
                    "a layout container cannot be laid out inside itself",
                ),
            ));
        }
    }
    diagnostics
}

fn containers_by_member(
    descriptors: &[ComponentDescriptor],
) -> BTreeMap<&ComponentId, Vec<&ComponentId>> {
    let mut containers = BTreeMap::<_, Vec<_>>::new();
    for descriptor in descriptors {
        let Some(layout) = descriptor.layout() else {
            continue;
        };
        for (member, _) in layout.fallback().members() {
            containers.entry(member).or_default().push(descriptor.id());
        }
    }
    containers
}

fn is_layout_cell(descriptor: &ComponentDescriptor) -> bool {
    matches!(
        descriptor.allocation_measurement_contract(),
        Some(ComponentAllocationMeasurementContract::LayoutCell(_))
    )
}

fn lays_out_itself(
    component: &ComponentId,
    containers: &BTreeMap<&ComponentId, Vec<&ComponentId>>,
) -> bool {
    let mut visited = BTreeSet::new();
    let mut frontier = containers.get(component).cloned().unwrap_or_default();
    while let Some(container) = frontier.pop() {
        if container == component {
            return true;
        }
        if visited.insert(container) {
            frontier.extend(containers.get(container).into_iter().flatten().copied());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::layout_membership_diagnostics;
    use crate::capability::{
        CapabilityDiagnosticCode, ComponentAllocationMeasurementContract, ComponentChildPolicy,
        ComponentDescriptor, ComponentId, ComponentPropSchema, ComponentStateOwnership,
        MosaicLayoutCell, MosaicLayoutContract, MosaicTrack,
    };

    fn id(value: &str) -> ComponentId {
        ComponentId::new(value).unwrap()
    }

    fn component(value: &str) -> ComponentDescriptor {
        ComponentDescriptor::new(
            id(value),
            ComponentPropSchema::named(format!("{value}.props")),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
    }

    fn cell(value: &str) -> ComponentDescriptor {
        component(value).with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::fill_layout_cell(),
        )
    }

    fn container(value: &str, members: &[&str]) -> ComponentDescriptor {
        let tracks = members
            .iter()
            .map(|_| MosaicTrack::flex(1, 0).unwrap())
            .collect::<Vec<_>>();
        let layout = members.iter().enumerate().fold(
            MosaicLayoutContract::columns(tracks).unwrap(),
            |layout, (column, member)| {
                layout
                    .with_member(id(member), MosaicLayoutCell::at(column as u16, 0))
                    .unwrap()
            },
        );
        component(value).with_layout(layout)
    }

    fn page(value: &str, members: &[&str]) -> ComponentDescriptor {
        container(value, members).with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::fill_viewport(),
        )
    }

    fn codes(descriptors: &[ComponentDescriptor]) -> Vec<(String, CapabilityDiagnosticCode)> {
        layout_membership_diagnostics(descriptors)
            .into_iter()
            .map(|(component, diagnostic)| (component.as_str().to_owned(), diagnostic.code()))
            .collect()
    }

    #[test]
    fn nested_containers_with_exclusive_cell_members_are_admitted() {
        let row = container(
            "demo.component.row",
            &["demo.component.a", "demo.component.b"],
        )
        .with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::fill_layout_cell(),
        );
        let page = page("demo.component.page", &["demo.component.row"]);
        let descriptors = [
            page,
            row,
            cell("demo.component.a"),
            cell("demo.component.b"),
        ];
        assert!(codes(&descriptors).is_empty());
    }

    #[test]
    fn orphaned_shared_uncelled_and_unregistered_members_are_rejected() {
        let descriptors = [
            page("demo.component.left", &["demo.component.shared"]),
            page("demo.component.right", &["demo.component.shared"]),
            cell("demo.component.shared"),
            cell("demo.component.orphan"),
            page("demo.component.loose", &["demo.component.free"]),
            component("demo.component.free"),
            page("demo.component.hollow", &["demo.component.unregistered"]),
        ];
        assert_eq!(
            codes(&descriptors),
            [
                (
                    "demo.component.shared".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
                (
                    "demo.component.orphan".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
                (
                    "demo.component.loose".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
                (
                    "demo.component.hollow".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
            ]
        );
    }

    #[test]
    fn portal_placed_members_and_unallocated_containers_are_rejected() {
        let descriptors = [
            container("demo.component.unallocated", &["demo.component.anchored"]),
            cell("demo.component.anchored").with_portal_child(
                crate::capability::ComponentPortalChildContract::new(id("demo.component.owner")),
            ),
        ];
        assert_eq!(
            codes(&descriptors),
            [
                (
                    "demo.component.unallocated".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
                (
                    "demo.component.anchored".to_owned(),
                    CapabilityDiagnosticCode::InvalidComponentLayoutMembership
                ),
            ]
        );
    }

    #[test]
    fn containers_laid_out_inside_themselves_are_rejected() {
        let cell_contract = ComponentAllocationMeasurementContract::fill_layout_cell();
        let descriptors = [
            container("demo.component.outer", &["demo.component.inner"])
                .with_allocation_measurement_contract(cell_contract),
            container("demo.component.inner", &["demo.component.outer"])
                .with_allocation_measurement_contract(cell_contract),
        ];
        assert_eq!(
            codes(&descriptors),
            [
                (
                    "demo.component.outer".to_owned(),
                    CapabilityDiagnosticCode::CyclicComponentLayout
                ),
                (
                    "demo.component.inner".to_owned(),
                    CapabilityDiagnosticCode::CyclicComponentLayout
                ),
            ]
        );
    }
}
