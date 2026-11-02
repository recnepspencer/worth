//! Preserve original composition custody while each owner retires its capability.
use crate::{
    world_section::ApplicationHomeBridgeComposition, ApplicationHomeCloseDenial as Denial,
};
use std::sync::{Arc, RwLock};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    BridgeHomeRetirementProgress as Progress, BridgeSealedRuntimeAssembly, RetiredRuntimeBridge,
    RetiredRuntimeBridgeRelationalSource, RetiringRuntimeBridge, RuntimeBridge,
    RuntimeBridgeRelationalSource,
};

pub(crate) enum HomeBridgeCloseCustody {
    Serving(ApplicationHomeBridgeComposition),
    Retiring(RetiringHomeBridgeComposition),
    Retired(RetiredHomeBridgeComposition),
}
pub(crate) struct RetiredHomeBridgeComposition {
    _allocation: RetiringHomeBridgeComposition,
}
pub(crate) struct RetiringHomeBridgeComposition {
    source: Option<RuntimeBridgeRelationalSource>,
    ordinary: Option<RuntimeBridge>,
    conditional_share: Option<Arc<RwLock<BridgeSealedRuntimeAssembly>>>,
    conditional: Option<RwLock<BridgeSealedRuntimeAssembly>>,
    ordinary_task: Option<RetiringRuntimeBridge>,
    _retired_ordinary: Option<RetiredRuntimeBridge>,
    _retired_source: Option<RetiredRuntimeBridgeRelationalSource>,
    stage: CompositionRetirementStage,
}
enum CompositionRetirementStage {
    Assembly,
    Ordinary,
    Source,
}

impl HomeBridgeCloseCustody {
    pub(crate) fn advance(
        self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(Self, Progress), (Self, Denial)> {
        match self {
            Self::Serving(original) => {
                let ApplicationHomeBridgeComposition {
                    source,
                    ordinary,
                    conditional,
                } = original;
                Ok((
                    Self::Retiring(RetiringHomeBridgeComposition {
                        source: Some(source),
                        ordinary: Some(ordinary),
                        conditional_share: Some(conditional),
                        conditional: None,
                        ordinary_task: None,
                        _retired_ordinary: None,
                        _retired_source: None,
                        stage: CompositionRetirementStage::Assembly,
                    }),
                    Progress::Advanced,
                ))
            }
            Self::Retiring(mut task) => match task.advance(request, native) {
                Ok(Progress::Advanced) => Ok((Self::Retiring(task), Progress::Advanced)),
                Ok(Progress::Complete) => Ok((
                    Self::Retired(RetiredHomeBridgeComposition { _allocation: task }),
                    Progress::Complete,
                )),
                Err(cause) => Err((Self::Retiring(task), cause)),
            },
            retired @ Self::Retired(_) => Ok((retired, Progress::Complete)),
        }
    }
    pub(crate) fn into_retired(self) -> Result<RetiredHomeBridgeComposition, Self> {
        match self {
            Self::Retired(retired) => Ok(retired),
            unfinished => Err(unfinished),
        }
    }
}

impl RetiringHomeBridgeComposition {
    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        match self.stage {
            CompositionRetirementStage::Assembly => {
                if let Some(conditional) = self.conditional_share.take() {
                    match Arc::try_unwrap(conditional) {
                        Ok(conditional) => self.conditional = Some(conditional),
                        Err(conditional) => {
                            self.conditional_share = Some(conditional);
                            return Err(Denial::WorldCompositionRetained);
                        }
                    }
                    return Ok(Progress::Advanced);
                }
                let conditional = self
                    .conditional
                    .as_mut()
                    .expect("same original owning assembly")
                    .get_mut()
                    .map_err(|_| Denial::WorldCompositionRetained)?;
                if conditional
                    .advance_home_retirement(request, native)
                    .map_err(Denial::Bridge)?
                    == Progress::Complete
                {
                    self.stage = CompositionRetirementStage::Ordinary;
                }
            }
            CompositionRetirementStage::Ordinary => {
                if let Some(ordinary) = self.ordinary.take() {
                    match ordinary.into_home_retirement(native, request) {
                        Ok(task) => self.ordinary_task = Some(task),
                        Err((ordinary, cause)) => {
                            self.ordinary = Some(ordinary);
                            return Err(Denial::Bridge(cause));
                        }
                    }
                    return Ok(Progress::Advanced);
                }
                if self
                    .ordinary_task
                    .as_mut()
                    .expect("same original Bridge task")
                    .advance(request, native)
                    .map_err(Denial::Bridge)?
                    == Progress::Complete
                {
                    self._retired_ordinary = Some(
                        self.ordinary_task
                            .take()
                            .expect("completed Bridge task")
                            .into_retired()
                            .unwrap_or_else(|_| unreachable!("owner reported Complete")),
                    );
                    self.stage = CompositionRetirementStage::Source;
                }
            }
            CompositionRetirementStage::Source => {
                let source = self
                    .source
                    .take()
                    .expect("same original composition source");
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => self._retired_source = Some(retired),
                    Err((source, cause)) => {
                        self.source = Some(source);
                        return Err(Denial::Bridge(
                            worth_runtime_bridge::facade::BridgeHomeRetirementDenial::NativeAlias(
                                cause,
                            ),
                        ));
                    }
                }
                return Ok(Progress::Complete);
            }
        }
        Ok(Progress::Advanced)
    }
}
