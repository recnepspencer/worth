//! The section privately lends its actual source and exact Serving owner.
use super::*;
impl ClosingRelationalSection {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        actual: &Arc<ServingPhysicalRuntime>,
    ) -> Result<RelationalSourceCloseProgress, RelationalSectionCloseDenial> {
        if self.complete {
            return Ok(RelationalSourceCloseProgress::Complete);
        }
        request
            .consult()
            .map_err(|cause| native_denial(RelationalSourceCloseDenial::Execution(cause)))?;
        if !self
            .serving
            .as_ref()
            .is_some_and(|original| Arc::ptr_eq(original, actual))
        {
            return Err(RelationalSectionCloseDenial::ForeignServing);
        }
        if let Some(runtime) = &mut self.runtime {
            let Some(ColdSourceCustody::Serving { source, .. }) = &self.cold_source else {
                unreachable!("Native callbacks precede source retirement");
            };
            if runtime
                .advance(request, source.as_ref())
                .map_err(native_denial)?
                == RelationalSourceCloseProgress::Complete
            {
                self.retired_runtime = Some(
                    self.runtime
                        .take()
                        .expect("same completed Native")
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("Native owner reported Complete")),
                );
            }
            return Ok(RelationalSourceCloseProgress::Advanced);
        }
        // Moving the original share into its own task never destroys its data.
        let original = self
            .cold_source
            .take()
            .expect("same original source phase remains owned");
        self.cold_source = Some(match original {
            ColdSourceCustody::Serving {
                source,
                preparation,
            } => match source.into_home_retirement(actual, preparation) {
                Ok(task) => ColdSourceCustody::Retiring(task),
                Err((source, preparation, cause)) => {
                    self.cold_source = Some(ColdSourceCustody::Serving {
                        source,
                        preparation,
                    });
                    return Err(RelationalSectionCloseDenial::Physical(cause));
                }
            },
            ColdSourceCustody::Retiring(mut task) => match task.advance(request, actual) {
                Ok(AnchoredRecordReadRetirementProgress::Advanced) => {
                    ColdSourceCustody::Retiring(task)
                }
                Ok(AnchoredRecordReadRetirementProgress::Complete) => ColdSourceCustody::Retired {
                    _original: task
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("source owner reported Complete")),
                },
                Err(cause) => {
                    self.cold_source = Some(ColdSourceCustody::Retiring(task));
                    return Err(RelationalSectionCloseDenial::Physical(cause));
                }
            },
            retired @ ColdSourceCustody::Retired { .. } => retired,
        });
        if matches!(self.cold_source, Some(ColdSourceCustody::Retired { .. })) {
            // Exact actual borrowing guarantees this is a nonfinal local Arc.
            self.serving.take();
            self.complete = true;
            return Ok(RelationalSourceCloseProgress::Complete);
        }
        Ok(RelationalSourceCloseProgress::Advanced)
    }
}
