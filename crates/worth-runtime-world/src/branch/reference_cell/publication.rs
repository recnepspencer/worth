use super::*;

impl ProductBranchReferenceCell {
    pub(crate) fn publish_recorded<A>(
        &self,
        expected: &ProductBranchObservation,
        argument: &mut A,
        materialize: impl FnOnce(&mut A) -> &mut Option<ProductBranchHeadProtection>,
        publication: crate::history::PreparedPublicationRecord,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        self.replace_expected(expected, argument, materialize, Some(publication))
    }

    pub(super) fn replace_expected<A>(
        &self,
        expected: &ProductBranchObservation,
        argument: &mut A,
        materialize: impl FnOnce(&mut A) -> &mut Option<ProductBranchHeadProtection>,
        publication: Option<crate::history::PreparedPublicationRecord>,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        let expected_snapshot = expected.snapshot();
        let mut current = self.state.write();
        if current.protection.is_none() {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::Retired,
                observed_head: current.snapshot.clone(),
            });
        }
        if &current.snapshot != expected_snapshot {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::ExpectedHeadMismatch(
                    expected
                        .mismatch_against_snapshot(&current.snapshot)
                        .expect("snapshot equality and observation comparison must agree"),
                ),
                observed_head: current.snapshot.clone(),
            });
        }
        if let Some(record) = publication.as_ref() {
            if let Err(denial) = record.check_cutoff() {
                return Err(ProductBranchReferenceLoss {
                    denial: ProductBranchReferenceCellDenial::Cutoff(denial),
                    observed_head: current.snapshot.clone(),
                });
            }
        }
        // Only an exactly current attempt may populate its reserved history
        // storage and transfer already bound World pin claims. The callback
        // must not allocate, contact a component owner, or touch this cell.
        let successor_slot = materialize(argument);
        #[cfg(test)]
        publication_unwind::after_materialized();
        let successor = successor_slot
            .as_ref()
            .expect("materialization retains complete custody");
        if let Err(denial) = validate_successor(&current.snapshot, successor) {
            return Err(ProductBranchReferenceLoss {
                denial,
                observed_head: current.snapshot.clone(),
            });
        }

        let successor_snapshot = successor.snapshot().clone();
        let Some(successor_receipt) = successor.transfer_receipt().cloned() else {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::SuccessorProtectionMismatch,
                observed_head: current.snapshot.clone(),
            });
        };
        let movement = ProductBranchReferenceMovement {
            before: current.snapshot.clone(),
            after: successor_snapshot.clone(),
            _retention_transfer: successor_receipt,
        };
        let publication = publication.map(|record| record.stage(&movement));
        let old_image = std::mem::replace(
            &mut *current,
            ProductBranchReferenceImage {
                snapshot: successor_snapshot,
                protection: Some(
                    successor_slot
                        .take()
                        .expect("validated successor custody moves only at the cell swap"),
                ),
            },
        );
        if let Some(publication) = publication {
            publication.mark_committed();
        }
        drop(current);
        #[cfg(test)]
        publication_unwind::after_committed();
        drop(old_image);
        Ok(movement)
    }
}
