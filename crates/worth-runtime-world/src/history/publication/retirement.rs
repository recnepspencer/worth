//! Retain original canonical facts while consuming each owner-issued basis.
use super::*;
use crate::{basis::NativeBasisRetirement, history::CompositeCommitRetirement};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial, runtime::RelationalRuntime,
};

pub(crate) struct PublicationFactsRetirement {
    shared: Option<Arc<CanonicalPublicationEnvelope>>,
    original: Option<CanonicalPublicationEnvelope>,
    commits: [Option<CompositeCommitRetirement>; 3],
    native_result: Option<NativeBasisRetirement>,
    step: usize,
    complete: bool,
}
impl PublicationFactsRetirement {
    pub(crate) fn new(shared: Arc<CanonicalPublicationEnvelope>) -> Self {
        Self {
            shared: Some(shared),
            original: None,
            commits: std::array::from_fn(|_| None),
            native_result: None,
            step: 0,
            complete: false,
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RelationalBranchBasisRetirementDenial> {
        if self.complete {
            return Ok(true);
        }
        if let Some(shared) = self.shared.take() {
            self.original = Arc::into_inner(shared);
            self.complete = self.original.is_none();
            return Ok(self.complete);
        }
        let original = self
            .original
            .as_mut()
            .expect("same original canonical envelope");
        if self.step < 3 {
            if self.commits[self.step].is_none() {
                let snapshot = if self.step == 0 {
                    &mut original.expected
                } else {
                    let (before, after) = original
                        .facts
                        .get_mut()
                        .expect("committed facts")
                        .movement
                        .retirement_snapshots();
                    if self.step == 1 {
                        before
                    } else {
                        after
                    }
                };
                self.commits[self.step] = Some(snapshot.begin_home_retirement());
                return Ok(false);
            }
            if self.commits[self.step]
                .as_mut()
                .expect("same original snapshot task")
                .advance(request, native)?
            {
                self.step += 1;
            }
            return Ok(false);
        }
        if self.step == 3 {
            self.native_result = original
                .facts
                .get_mut()
                .expect("committed facts")
                .component_results
                .take_native_for_retirement()
                .map(NativeBasisRetirement::new);
            self.step += 1;
            return Ok(false);
        }
        self.complete = match &mut self.native_result {
            Some(task) => task.advance(request, native)?,
            None => true,
        };
        Ok(self.complete)
    }
}
