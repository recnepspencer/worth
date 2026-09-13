use crate::capability::{ComponentDescriptor, ComponentFocusSupport, ComponentHitTestContract};

use super::digest::{fold, fold_text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiComponentPlanMeaning {
    descriptor: ComponentDescriptor,
    child_range_identity: Option<String>,
}

impl WorthUiComponentPlanMeaning {
    pub(crate) fn new(
        descriptor: ComponentDescriptor,
        child_range_identity: Option<String>,
    ) -> Self {
        Self {
            descriptor,
            child_range_identity,
        }
    }

    pub(crate) fn child_range_identity(&self) -> Option<&str> {
        self.child_range_identity.as_deref()
    }

    pub(crate) fn same_mounted_layout_meaning(&self, successor: &Self) -> bool {
        self.descriptor == successor.descriptor
    }

    pub(crate) fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }

    pub(crate) fn surface_paint_order(&self) -> Option<u32> {
        self.descriptor.surface_paint_order()
    }

    pub(crate) fn semantic_text_layer_order(&self) -> Option<u32> {
        Some(
            self.descriptor
                .semantic_text_contract()?
                .layer_semantic_order(),
        )
    }

    pub(crate) fn semantic_text_contract(
        &self,
    ) -> Option<&crate::capability::ComponentSemanticTextContract> {
        self.descriptor.semantic_text_contract()
    }

    pub(crate) fn hit_test_contract(&self) -> Option<ComponentHitTestContract> {
        self.descriptor.hit_test_contract()
    }

    pub(crate) fn focus_support(&self) -> ComponentFocusSupport {
        self.descriptor.focus()
    }

    pub(crate) fn portal_child_owner(&self) -> Option<&crate::capability::ComponentId> {
        self.descriptor
            .portal_child_contract()
            .map(crate::capability::ComponentPortalChildContract::owner)
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let digest = self.descriptor.theme_token_dependencies().iter().fold(
            fold_text(0x636f_6d70_6f6e_656e, self.descriptor.id().as_str()),
            |digest, token| fold_text(fold(digest, 1), token.as_str()),
        );
        let digest = match self.surface_paint_order() {
            Some(rank) => fold(fold(digest, 0x7375_7266_6f72_6401), u64::from(rank)),
            None => fold(digest, 0x7375_7266_6f72_6400),
        };
        let digest = self
            .semantic_text_layer_order()
            .map_or(digest, |order| fold(digest, u64::from(order)));
        let digest = self.hit_test_contract().map_or(digest, |contract| {
            fold(
                fold(digest, 0x6869_745f_7465_7374),
                u64::from(contract.order().rank()),
            )
        });
        let digest = fold_text(digest, self.focus_support().as_str());
        self.portal_child_owner().map_or(digest, |owner| {
            fold_text(fold(digest, 0x706f_7274_616c_6368), owner.as_str())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::capability::{
        ComponentChildPolicy, ComponentDescriptor, ComponentFocusSupport, ComponentId,
        ComponentPropSchema, ComponentStateOwnership,
    };

    use super::WorthUiComponentPlanMeaning;

    #[test]
    fn focus_support_survives_into_semantic_plan_meaning() {
        let plain = meaning(component());
        let focusable = meaning(component().with_focus(ComponentFocusSupport::focusable()));
        assert_eq!(
            plain.focus_support(),
            ComponentFocusSupport::not_focusable()
        );
        assert_eq!(
            focusable.focus_support(),
            ComponentFocusSupport::focusable()
        );
        assert_ne!(plain.semantic_digest(), focusable.semantic_digest());
    }

    #[test]
    fn surface_paint_order_is_executable_meaning_without_static_paint_inference() {
        let legacy = component();
        let variants = [None, Some(0), Some(65_536), Some(u32::MAX)].map(|rank| {
            let descriptor = rank.map_or_else(
                || legacy.clone(),
                |rank| legacy.clone().with_surface_paint_order(rank),
            );
            let meaning = meaning(descriptor);
            assert_eq!(meaning.surface_paint_order(), rank);
            meaning
        });
        for (index, left) in variants.iter().enumerate() {
            for right in &variants[index + 1..] {
                assert_ne!(left.semantic_digest(), right.semantic_digest());
            }
        }
        assert_eq!(
            meaning(component().with_surface_paint_order(23)).surface_paint_order(),
            Some(23),
        );
    }

    fn component() -> ComponentDescriptor {
        ComponentDescriptor::new(
            ComponentId::new("workspace.component.pulse").expect("valid component id"),
            ComponentPropSchema::named("workspace.component.pulse.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
    }

    fn meaning(descriptor: ComponentDescriptor) -> WorthUiComponentPlanMeaning {
        WorthUiComponentPlanMeaning::new(descriptor, None)
    }
}
