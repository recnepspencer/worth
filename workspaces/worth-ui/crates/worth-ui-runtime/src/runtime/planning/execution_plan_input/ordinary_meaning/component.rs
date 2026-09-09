use crate::capability::{
    ComponentDescriptor, ComponentFocusSupport, ComponentHitTestContract,
    ComponentStaticPaintContract, ComponentStaticPaintOrder, ThemeTokenId,
};

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

    pub(crate) fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }

    pub(crate) fn static_paint_theme_token_dependency(&self) -> Option<&ThemeTokenId> {
        Some(self.static_paint_contract()?.theme_token())
    }

    pub(crate) fn static_paint_order(&self) -> Option<ComponentStaticPaintOrder> {
        Some(self.static_paint_contract()?.order())
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

    fn static_paint_contract(&self) -> Option<&ComponentStaticPaintContract> {
        self.descriptor.static_paint_contract()
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let digest = self.descriptor.theme_token_dependencies().iter().fold(
            fold_text(0x636f_6d70_6f6e_656e, self.descriptor.id().as_str()),
            |digest, token| fold_text(fold(digest, 1), token.as_str()),
        );
        let digest = self
            .static_paint_order()
            .map_or(digest, |order| fold(digest, u64::from(order.rank())));
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

    #[cfg(test)]
    pub(crate) fn with_static_paint_order_for_test(rank: u32) -> Self {
        let descriptor = ComponentDescriptor::new(
            crate::capability::ComponentId::new("workspace.component.paint")
                .expect("valid component id"),
            crate::capability::ComponentPropSchema::named("workspace.component.paint.props"),
            crate::capability::ComponentChildPolicy::no_children(),
            crate::capability::ComponentStateOwnership::runtime_owned(),
        )
        .with_static_paint(
            ComponentStaticPaintContract::opaque_fill(
                crate::capability::ThemeTokenId::new("theme.paint.fill").expect("valid token id"),
                ComponentStaticPaintOrder::back_to_front(rank),
            ),
            crate::capability::ComponentAllocationMeasurementContract::fill_viewport(),
        );
        Self::new(descriptor, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::capability::{
        ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
        ComponentFocusSupport, ComponentId, ComponentPropSchema, ComponentStateOwnership,
        ComponentStaticPaintContract, ComponentStaticPaintOrder, ComponentViewportInset,
        ThemeTokenId,
    };

    use super::WorthUiComponentPlanMeaning;

    #[test]
    fn static_paint_requires_an_explicit_complete_contract() {
        let token = ThemeTokenId::new("theme.pulse.fill").expect("valid token id");
        let generic = meaning(component().with_theme_token_dependency(token.clone()));
        let inferred = meaning(
            component()
                .with_theme_token_dependency(token.clone())
                .with_allocation_measurement_contract(
                    ComponentAllocationMeasurementContract::fill_viewport(),
                ),
        );
        let viewport = meaning(component().with_static_paint(
            ComponentStaticPaintContract::opaque_fill(
                token.clone(),
                ComponentStaticPaintOrder::back_to_front(0),
            ),
            ComponentAllocationMeasurementContract::fill_viewport(),
        ));
        let inset = meaning(component().with_static_paint(
            ComponentStaticPaintContract::opaque_fill(
                token.clone(),
                ComponentStaticPaintOrder::back_to_front(1),
            ),
            ComponentAllocationMeasurementContract::viewport_inset(
                ComponentViewportInset::symmetric(48, 24),
            ),
        ));

        assert_eq!(generic.static_paint_theme_token_dependency(), None);
        assert_eq!(inferred.static_paint_theme_token_dependency(), None);
        assert_eq!(viewport.static_paint_theme_token_dependency(), Some(&token));
        assert_eq!(inset.static_paint_theme_token_dependency(), Some(&token));
        assert_eq!(
            viewport.static_paint_order(),
            Some(ComponentStaticPaintOrder::back_to_front(0))
        );
        assert_eq!(
            inset.static_paint_order(),
            Some(ComponentStaticPaintOrder::back_to_front(1))
        );
    }

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
        let legacy = WorthUiComponentPlanMeaning::with_static_paint_order_for_test(7)
            .descriptor()
            .clone();
        assert_eq!(meaning(legacy.clone()).surface_paint_order(), None);
        let variants = [None, Some(0), Some(65_536), Some(u32::MAX)].map(|rank| {
            let descriptor = rank.map_or_else(
                || legacy.clone(),
                |rank| legacy.clone().with_surface_paint_order(rank),
            );
            let meaning = meaning(descriptor);
            assert_eq!(meaning.surface_paint_order(), rank);
            assert_eq!(meaning.static_paint_order().unwrap().rank(), 7);
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
