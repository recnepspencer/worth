use super::{
    WorthQueryArtifactDenial, WorthQueryDisposedArtifact, WorthQueryMoveOnlyArtifactHandle,
};

#[derive(Debug)]
pub struct WorthQueryReplacedArtifact {
    prior: WorthQueryDisposedArtifact,
    replacement: WorthQueryMoveOnlyArtifactHandle,
}

impl WorthQueryReplacedArtifact {
    pub fn new(
        prior: WorthQueryDisposedArtifact,
        replacement: WorthQueryMoveOnlyArtifactHandle,
    ) -> Self {
        Self { prior, replacement }
    }

    pub fn prior(&self) -> &WorthQueryDisposedArtifact {
        &self.prior
    }

    pub fn replacement(&self) -> &WorthQueryMoveOnlyArtifactHandle {
        &self.replacement
    }

    pub fn into_replacement(self) -> WorthQueryMoveOnlyArtifactHandle {
        self.replacement
    }

    pub fn into_parts(self) -> (WorthQueryDisposedArtifact, WorthQueryMoveOnlyArtifactHandle) {
        (self.prior, self.replacement)
    }
}

#[derive(Debug)]
pub struct WorthQueryArtifactReplacementStop {
    denial: WorthQueryArtifactDenial,
    retained: WorthQueryMoveOnlyArtifactHandle,
}

impl WorthQueryArtifactReplacementStop {
    pub fn new(
        denial: WorthQueryArtifactDenial,
        retained: WorthQueryMoveOnlyArtifactHandle,
    ) -> Self {
        Self { denial, retained }
    }

    pub fn denial(&self) -> &WorthQueryArtifactDenial {
        &self.denial
    }

    pub fn retained(&self) -> &WorthQueryMoveOnlyArtifactHandle {
        &self.retained
    }

    pub fn into_retained(self) -> WorthQueryMoveOnlyArtifactHandle {
        self.retained
    }

    pub fn into_parts(self) -> (WorthQueryArtifactDenial, WorthQueryMoveOnlyArtifactHandle) {
        (self.denial, self.retained)
    }
}
