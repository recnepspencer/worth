use std::rc::Rc;
use worth_ui_host_contract::{
    UiHostPointerIdentity, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

/// Exact semantic dependencies for pointer output reuse. Receipt, observation,
/// and input sequence changes do not alter the physical mechanic.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPointerAffordanceReuseBasis(Option<Rc<[PointerMechanicDependency]>>);

#[derive(Clone, Debug, Eq, PartialEq)]
struct PointerMechanicDependency {
    surface: UiSemanticSurfaceIdentity,
    pointer: UiHostPointerIdentity,
    target: UiMountedInstanceIdentity,
    family: crate::declaration::UiPointerAffordance,
}

impl UiPointerAffordanceReuseBasis {
    pub(crate) fn from_snapshot(
        snapshot: Option<&super::UiPointerAffordanceSnapshot>,
        includes_surface: impl Fn(UiSemanticSurfaceIdentity) -> bool,
    ) -> Self {
        let mut rows = snapshot
            .into_iter()
            .flat_map(|snapshot| snapshot.active_projections())
            .filter(|row| includes_surface(row.surface()))
            .filter_map(|row| {
                Some(PointerMechanicDependency {
                    surface: row.surface(),
                    pointer: row.pointer(),
                    target: row.target()?,
                    family: row.family(),
                })
            })
            .collect::<Vec<_>>();
        rows.sort_unstable_by_key(|row| row.surface);
        Self((!rows.is_empty()).then(|| rows.into()))
    }
}
