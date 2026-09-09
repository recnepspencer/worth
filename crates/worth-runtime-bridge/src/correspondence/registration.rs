use super::{
    BridgeCorrespondenceDenial, BridgeCorrespondenceDenialKind, BridgeSemanticDependencyCandidate,
    BridgeSignalAspectTargetDeclaration,
};

/// Frozen volatile lowering registration for one exact source-installed
/// semantic dependency. Signal layout is deliberately absent from the source
/// meaning and is selected only at runtime construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSemanticCorrespondenceRegistration {
    pub(crate) dependency: BridgeSemanticDependencyCandidate,
    pub(crate) targets: Vec<BridgeSignalAspectTargetDeclaration>,
}

impl BridgeSemanticCorrespondenceRegistration {
    pub fn new(
        dependency: BridgeSemanticDependencyCandidate,
        mut targets: Vec<BridgeSignalAspectTargetDeclaration>,
    ) -> Result<Self, BridgeCorrespondenceDenial> {
        if targets.is_empty() {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::EmptyTargetSet,
            ));
        }
        let graph = targets[0].graph_instance_id();
        if targets
            .iter()
            .any(|target| target.graph_instance_id() != graph)
        {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::MixedGraphTargetSet,
            ));
        }
        targets.sort_by_key(BridgeSignalAspectTargetDeclaration::canonical_registration_key);
        if targets.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::DuplicateTarget,
            ));
        }
        Ok(Self {
            dependency,
            targets,
        })
    }

    pub fn dependency(&self) -> &BridgeSemanticDependencyCandidate {
        &self.dependency
    }

    pub(crate) fn canonical_key(&self) -> String {
        self.dependency.canonical_registration_key()
    }

    pub(crate) fn signal_graph_instance_id(&self) -> u64 {
        self.targets[0].graph_instance_id()
    }

    pub(crate) fn has_new_targets(&self, extension: &Self) -> bool {
        extension.targets.iter().any(|candidate| {
            self.targets
                .binary_search_by_key(
                    &candidate.canonical_registration_key(),
                    BridgeSignalAspectTargetDeclaration::canonical_registration_key,
                )
                .is_err()
        })
    }

    pub(crate) fn extend_targets(
        &mut self,
        extension: &Self,
    ) -> Result<(), BridgeCorrespondenceDenial> {
        if self.dependency != extension.dependency
            || self.signal_graph_instance_id() != extension.signal_graph_instance_id()
        {
            return Err(BridgeCorrespondenceDenial::without_admission(
                BridgeCorrespondenceDenialKind::InvalidPortableDependency,
            ));
        }
        for target in &extension.targets {
            let key = target.canonical_registration_key();
            match self.targets.binary_search_by_key(
                &key,
                BridgeSignalAspectTargetDeclaration::canonical_registration_key,
            ) {
                Ok(_) => {}
                Err(position) => self.targets.insert(position, target.clone()),
            }
        }
        Ok(())
    }
}
