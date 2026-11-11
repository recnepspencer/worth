//! Home retains the same prepared initial source through request stops and cleanup.
use super::*;
use worth_signal::facade::durability::{
    SignalInitialSourceProgress, SignalInitialSourcePublication, SignalInitialSourceStepRefusal,
};
#[derive(Debug)]
pub enum HomeSignalInitialSourceDenial {
    Medium(D),
    Step(SignalInitialSourceStepRefusal),
    NotPublished,
    Transferred,
}
pub(crate) trait OriginalSignalInitialSource: Send {
    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, SignalInitialSourceStepRefusal>;
}
impl<Dm, I, E, Ctx, T> OriginalSignalInitialSource
    for SignalInitialSourcePublication<Dm, I, E, Ctx, T>
where
    Dm: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, SignalInitialSourceStepRefusal> {
        SignalInitialSourcePublication::advance(self, request)
    }
}
impl OriginalSignalInitialSource for worth_runtime_bridge::facade::BridgeInitialSourcePublication {
    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, SignalInitialSourceStepRefusal> {
        worth_runtime_bridge::facade::BridgeInitialSourcePublication::advance(self, request)
    }
}
enum InitialSourceState<T> {
    Driving(T),
    Published(T),
    Transferred,
}
pub(crate) struct HomeInitialSignalSource<T> {
    state: Mutex<InitialSourceState<T>>,
    _funding: Option<ExecutionMemoryReservation>,
}
pub(super) trait RetainedInitialSignalSource: Send + Sync {
    fn settle_for_release(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), HomeSignalInitialSourceDenial>;
}
impl<T: OriginalSignalInitialSource> HomeInitialSignalSource<T> {
    pub(super) fn retain(original: T, request: &ExecutionRequest<'_, '_>) -> Result<Arc<Self>, D> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed original-source layout")
            .0
            .pad_to_align();
        let funding = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(D::Request)?;
        Ok(Arc::new(Self {
            state: Mutex::new(InitialSourceState::Driving(original)),
            _funding: funding,
        }))
    }
    pub(crate) fn advance(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, HomeSignalInitialSourceDenial> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| HomeSignalInitialSourceDenial::Medium(D::PublicationBusy))?;
        let answer = match &mut *state {
            InitialSourceState::Driving(original) => original
                .advance(request)
                .map_err(HomeSignalInitialSourceDenial::Step)?,
            InitialSourceState::Published(_) => return Ok(SignalInitialSourceProgress::Published),
            InitialSourceState::Transferred => {
                return Err(HomeSignalInitialSourceDenial::Transferred)
            }
        };
        match answer {
            SignalInitialSourceProgress::Pending => Ok(answer),
            SignalInitialSourceProgress::Published => {
                let InitialSourceState::Driving(original) =
                    std::mem::replace(&mut *state, InitialSourceState::Transferred)
                else {
                    unreachable!("same original drive")
                };
                *state = InitialSourceState::Published(original);
                Ok(answer)
            }
        }
    }
    pub(crate) fn take_published(&self) -> Result<T, HomeSignalInitialSourceDenial> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| HomeSignalInitialSourceDenial::Medium(D::PublicationBusy))?;
        match std::mem::replace(&mut *state, InitialSourceState::Transferred) {
            InitialSourceState::Published(original) => Ok(original),
            InitialSourceState::Driving(original) => {
                *state = InitialSourceState::Driving(original);
                Err(HomeSignalInitialSourceDenial::NotPublished)
            }
            InitialSourceState::Transferred => Err(HomeSignalInitialSourceDenial::Transferred),
        }
    }
}
impl<T: OriginalSignalInitialSource> RetainedInitialSignalSource for HomeInitialSignalSource<T> {
    fn settle_for_release(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), HomeSignalInitialSourceDenial> {
        loop {
            match self.advance(request) {
                Ok(SignalInitialSourceProgress::Pending) => request
                    .consult()
                    .map_err(|cause| HomeSignalInitialSourceDenial::Medium(D::Request(cause)))?,
                Ok(SignalInitialSourceProgress::Published) => {
                    drop(self.take_published()?);
                    return Ok(());
                }
                Err(HomeSignalInitialSourceDenial::Transferred) => return Ok(()),
                Err(cause) => return Err(cause),
            }
        }
    }
}
