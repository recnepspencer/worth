//! Encoding controls retain the first refusal, including a swallowed callback error.
use super::{CanonicalEncodeError, CanonicalEncodingCharge};
use std::{convert::Infallible, fmt::Debug, marker::PhantomData};

pub trait EncodingAdmission {
    type Error: Debug;
    fn tracks_work(&self) -> bool {
        true
    }
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<Self::Error>> {
        Ok(())
    }
    fn admit(
        &mut self,
        charge: CanonicalEncodingCharge,
    ) -> Result<(), CanonicalEncodeError<Self::Error>>;
    fn has_failure(&self) -> bool;
    fn take_failure(&mut self) -> Option<Self::Error>;
}

pub struct NoAdmission;
impl EncodingAdmission for NoAdmission {
    type Error = Infallible;
    fn admit(
        &mut self,
        _: CanonicalEncodingCharge,
    ) -> Result<(), CanonicalEncodeError<Infallible>> {
        Ok(())
    }
    fn has_failure(&self) -> bool {
        false
    }
    fn take_failure(&mut self) -> Option<Infallible> {
        None
    }
}

pub struct CallbackAdmission<'a, F, E> {
    callback: &'a mut F,
    marker: PhantomData<fn() -> E>,
    failure: Option<E>,
}
impl<'a, F, E> CallbackAdmission<'a, F, E> {
    pub fn new(callback: &'a mut F) -> Self {
        Self {
            callback,
            marker: PhantomData,
            failure: None,
        }
    }
}
impl<F, E: Debug> EncodingAdmission for CallbackAdmission<'_, F, E>
where
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
{
    type Error = E;
    fn admit(&mut self, charge: CanonicalEncodingCharge) -> Result<(), CanonicalEncodeError<E>> {
        if self.failure.is_some() {
            return Err(CanonicalEncodeError::AdmissionDeferred);
        }
        match (self.callback)(charge) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.failure = Some(error);
                Err(CanonicalEncodeError::AdmissionDeferred)
            }
        }
    }
    fn has_failure(&self) -> bool {
        self.failure.is_some()
    }
    fn take_failure(&mut self) -> Option<E> {
        self.failure.take()
    }
}

pub struct PollingAdmission<'a, F, E> {
    callback: &'a mut F,
    failure: Option<E>,
}
impl<'a, F, E> PollingAdmission<'a, F, E> {
    pub fn new(callback: &'a mut F) -> Self {
        Self {
            callback,
            failure: None,
        }
    }
}
impl<F, E: Debug> EncodingAdmission for PollingAdmission<'_, F, E>
where
    F: FnMut() -> Result<(), E>,
{
    type Error = E;
    fn tracks_work(&self) -> bool {
        false
    }
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<E>> {
        if self.failure.is_some() {
            return Err(CanonicalEncodeError::AdmissionDeferred);
        }
        match (self.callback)() {
            Ok(()) => Ok(()),
            Err(error) => {
                self.failure = Some(error);
                Err(CanonicalEncodeError::AdmissionDeferred)
            }
        }
    }
    fn admit(&mut self, _: CanonicalEncodingCharge) -> Result<(), CanonicalEncodeError<E>> {
        unreachable!("polling-only encoding must select controls before calculating a charge")
    }
    fn has_failure(&self) -> bool {
        self.failure.is_some()
    }
    fn take_failure(&mut self) -> Option<E> {
        self.failure.take()
    }
}
