//! The framework lends member execution only from an advancement's wave.
use super::{Form, QueryRequestExecution};

pub struct WaveRequestExecution<'authority> {
    execution: QueryRequestExecution<'authority>,
}
pub(in crate::domain_computation::primary_graph) struct MemberRequestExecution<'authority> {
    execution: QueryRequestExecution<'authority>,
}
impl<'authority> QueryRequestExecution<'authority> {
    fn descendant(&self) -> Self {
        let form = match &self.form {
            Form::Leased { lease, policy } | Form::Serial { lease, policy } => Form::Leased {
                lease: lease
                    .as_ref()
                    .map(|lease| {
                        lease.controlled_child(worth_execution::CancellationToken::new(), None)
                    })
                    .map_err(|denial| *denial),
                policy: *policy,
            },
        };
        Self {
            cancellation: self.cancellation.clone(),
            deadline: self.deadline,
            declared_work: self.declared_work,
            certified_seed: self.certified_seed,
            form,
        }
    }
    pub fn wave(
        &self,
    ) -> WaveRequestExecution<'authority> {
        WaveRequestExecution {
            execution: self.descendant(),
        }
    }
}
impl<'authority> WaveRequestExecution<'authority> {
    pub fn request(
        &self,
    ) -> &QueryRequestExecution<'_> {
        &self.execution
    }

    pub fn member(
        &self,
    ) -> MemberRequestExecution<'authority> {
        MemberRequestExecution {
            execution: self.execution.descendant(),
        }
    }
}
impl MemberRequestExecution<'_> {
    pub fn request(
        &self,
    ) -> &QueryRequestExecution<'_> {
        &self.execution
    }
}
