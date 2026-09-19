use worth_runtime_world::facade::ProductUnpublishedOwnerEffects;

#[derive(Debug)]
pub struct WorthQueryUnpublishedBranchAdoption {
    owner_effects: ProductUnpublishedOwnerEffects,
}

impl WorthQueryUnpublishedBranchAdoption {
    pub(super) fn new(owner_effects: ProductUnpublishedOwnerEffects) -> Self {
        Self { owner_effects }
    }

    pub fn into_owner_effects(self) -> ProductUnpublishedOwnerEffects {
        self.owner_effects
    }
}
