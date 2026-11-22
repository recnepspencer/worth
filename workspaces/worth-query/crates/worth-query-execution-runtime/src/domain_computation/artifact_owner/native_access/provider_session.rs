use worth_query_installation::facade::WorthQueryArtifactNativeLayoutReference;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactProviderAccessSession {
    identity: String,
    generation: u64,
    borrow_generation: u64,
    layout: WorthQueryArtifactNativeLayoutReference,
}

impl WorthQueryArtifactProviderAccessSession {
    pub fn mint(
        identity: String,
        generation: u64,
        borrow_generation: u64,
        layout: WorthQueryArtifactNativeLayoutReference,
    ) -> Self {
        Self {
            identity,
            generation,
            borrow_generation,
            layout,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn borrow_generation(&self) -> u64 {
        self.borrow_generation
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }
}
