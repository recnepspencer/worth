//! A borrowed incoming view cannot manufacture capture or current-output custody.
use super::super::execution::CapturedProducerInputRead;
use super::*;

pub struct PrerequisiteInputs<'a>(pub(super) InputState<'a>);
#[derive(Clone, Copy)]
pub(super) enum InputState<'a> {
    Stored,
    Ready,
    Captured(&'a dyn CapturedProducerInputRead),
}
impl<'a> From<Option<&'a super::super::super::coupled_publication::PreparedCoupledInputs>>
    for PrerequisiteInputs<'a>
{
    fn from(
        inputs: Option<&'a super::super::super::coupled_publication::PreparedCoupledInputs>,
    ) -> Self {
        match inputs {
            None => Self(InputState::Stored),
            Some(super::super::super::coupled_publication::PreparedCoupledInputs::Ready(_)) => {
                Self(InputState::Ready)
            }
            Some(super::super::super::coupled_publication::PreparedCoupledInputs::Captured(
                inputs,
            )) => Self(InputState::Captured(inputs.as_ref())),
        }
    }
}
