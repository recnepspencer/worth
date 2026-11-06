//! Retire Query's ordinary Bridge and its local alias to Store's assembly.
use super::{InstalledBridgeCustody, WorthQueryInstalledApplicationBridge};
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryOperationHomeRetirementProgress as Progress,
};
use std::sync::{Arc, RwLock};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    BridgeHomeRetirementDenial as BridgeDenial, BridgeHomeRetirementProgress as BridgeProgress,
    BridgeSealedRuntimeAssembly,
};

impl WorthQueryInstalledApplicationBridge {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.ordinary()
            .preflight_home_retirement(native, request)
            .map_err(Denial::Bridge)?;
        self.conditional_owner()
            .expect("same original assembly")
            .try_write()
            .map_err(|_| Denial::Bridge(BridgeDenial::Busy))?
            .preflight_home_retirement(request, native)
            .map_err(Denial::Bridge)
    }

    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        owner: &Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Result<Progress, Denial> {
        let Some(conditional) = self.conditional_owner() else {
            return Ok(Progress::Complete);
        };
        request.consult().map_err(Denial::Execution)?;
        if !Arc::ptr_eq(conditional, owner) {
            return Err(Denial::Bridge(BridgeDenial::ForeignOwner));
        }
        let original = self.custody.take().expect("same original Bridge phase");
        let (successor, outcome) = match original {
            InstalledBridgeCustody::Serving {
                ordinary,
                conditional,
            } => match ordinary.into_home_retirement(native, request) {
                Ok(task) => (
                    InstalledBridgeCustody::Retiring {
                        conditional,
                        ordinary: task,
                    },
                    Ok(Progress::Advanced),
                ),
                Err((ordinary, cause)) => (
                    InstalledBridgeCustody::Serving {
                        ordinary,
                        conditional,
                    },
                    Err(Denial::Bridge(cause)),
                ),
            },
            InstalledBridgeCustody::Retiring {
                conditional,
                mut ordinary,
            } => match ordinary.advance(request, native) {
                Ok(BridgeProgress::Advanced) => (
                    InstalledBridgeCustody::Retiring {
                        conditional,
                        ordinary,
                    },
                    Ok(Progress::Advanced),
                ),
                Err(cause) => (
                    InstalledBridgeCustody::Retiring {
                        conditional,
                        ordinary,
                    },
                    Err(Denial::Bridge(cause)),
                ),
                Ok(BridgeProgress::Complete) => {
                    let ordinary = ordinary
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("owner reported Complete"));
                    (
                        InstalledBridgeCustody::Retired {
                            conditional,
                            ordinary,
                        },
                        Ok(Progress::Advanced),
                    )
                }
            },
            InstalledBridgeCustody::Retired {
                conditional,
                ordinary,
            } => {
                // Store's matching Arc keeps the assembly alive through release.
                drop(conditional);
                (
                    InstalledBridgeCustody::Released { ordinary },
                    Ok(Progress::Complete),
                )
            }
            InstalledBridgeCustody::Released { ordinary } => (
                InstalledBridgeCustody::Released { ordinary },
                Ok(Progress::Complete),
            ),
        };
        self.custody = Some(successor);
        outcome
    }
}
