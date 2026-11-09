//! Exact installation affinity is distinct from the programme's shared sources.
use std::sync::Arc;
#[derive(Debug)]
pub struct InstalledBridgeProgramOwner {
    occurrence: Arc<()>,
}
#[derive(Clone, Debug)]
pub struct BridgeProgramInstallationBinding {
    occurrence: Arc<()>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeProgramInstallationDenial {
    ForeignInstallation,
}
impl InstalledBridgeProgramOwner {
    pub fn fresh() -> Self {
        Self {
            occurrence: Arc::new(()),
        }
    }
    pub fn binding(&self) -> BridgeProgramInstallationBinding {
        BridgeProgramInstallationBinding {
            occurrence: Arc::clone(&self.occurrence),
        }
    }
}
impl BridgeProgramInstallationBinding {
    pub fn admit(&self, candidate: &Self) -> Result<(), BridgeProgramInstallationDenial> {
        if Arc::ptr_eq(&self.occurrence, &candidate.occurrence) {
            Ok(())
        } else {
            Err(BridgeProgramInstallationDenial::ForeignInstallation)
        }
    }
}
