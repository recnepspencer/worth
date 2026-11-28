use super::*;

impl ProductBranchReferenceCell {
    pub(crate) fn publish_recorded<A>(
        &self,
        expected: &ProductBranchObservation,
        candidate: &ProductBranchReferenceSnapshot,
        argument: &mut A,
        materialize: impl FnOnce(&mut A) -> &mut Option<ProductBranchHeadProtection>,
        publication: crate::history::PreparedPublicationRecord,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        self.replace_expected_permitted(
            expected,
            candidate,
            argument,
            materialize,
            Some(publication),
            None,
        )
    }

    pub(super) fn replace_expected_permitted<A>(
        &self,
        expected: &ProductBranchObservation,
        candidate: &ProductBranchReferenceSnapshot,
        argument: &mut A,
        materialize: impl FnOnce(&mut A) -> &mut Option<ProductBranchHeadProtection>,
        publication: Option<crate::history::PreparedPublicationRecord>,
        permit: Option<&WorldProductMovementPermit>,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        let expected_snapshot = expected.snapshot();
        let mut current = self.state.write();
        match permit {
            Some(permit) if permit.admits(&current.movement) => {}
            Some(_) => {
                return Err(ProductBranchReferenceLoss {
                    denial: ProductBranchReferenceCellDenial::RecoveryRequired,
                    observed_head: current.snapshot.clone(),
                })
            }
            None => {
                if let Some(denial) = current.movement.denial() {
                    return Err(ProductBranchReferenceLoss {
                        denial,
                        observed_head: current.snapshot.clone(),
                    });
                }
            }
        }
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
        if permit.is_none()
            && let Some(record) = publication.as_ref()
            && let Err(denial) = record.check_cutoff()
        {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::Cutoff(denial),
                observed_head: current.snapshot.clone(),
            });
        }
        if let Err(denial) =
            super::successor_validation::validate_successor_snapshot(&current.snapshot, candidate)
        {
            return Err(ProductBranchReferenceLoss {
                denial,
                observed_head: current.snapshot.clone(),
            });
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
        if successor.snapshot() != candidate {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::SuccessorProtectionMismatch,
                observed_head: current.snapshot.clone(),
            });
        }
        let successor_admission = match permit {
            Some(permit) => permit.validate_occurrence_successor(&current.snapshot, successor),
            None => validate_successor(&current.snapshot, successor),
        };
        if let Err(denial) = successor_admission {
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
        if permit.is_some_and(WorldProductMovementPermit::refuse_head_movement) {
            return Err(ProductBranchReferenceLoss {
                denial: ProductBranchReferenceCellDenial::ControlledInterruption,
                observed_head: current.snapshot.clone(),
            });
        }
        let seated = current
            .protection
            .as_ref()
            .is_some_and(ProductBranchHeadProtection::history_is_seated);
        successor_slot
            .as_mut()
            .expect("successor stays in original custody through history seating")
            .seat_history(seated);
        let acknowledged_subject = permit.map(|_| current.movement.consume_committed());
        let successor = successor_slot
            .take()
            .expect("validated successor custody moves only at the cell swap");
        let old_image = std::mem::replace(
            &mut *current,
            ProductBranchReferenceImage {
                movement: ProductMovementPhase::Idle,
                snapshot: successor_snapshot,
                protection: Some(successor),
            },
        );
        if let Some(publication) = publication {
            publication.mark_committed();
        }
        drop(current);
        drop(acknowledged_subject);
        if let Some(permit) = permit {
            permit.after_committed();
        }
        #[cfg(test)]
        publication_unwind::after_committed();
        drop(old_image);
        Ok(movement)
    }

    /// Replace only if the complete expected observation is still selected.
    /// Expected-currentness, successor validation, and pair replacement use
    /// one short branch-local lock; no owner call occurs while it is held.
    #[cfg(test)]
    pub(crate) fn compare_and_publish(
        &self,
        expected: &ProductBranchObservation,
        successor: ProductBranchHeadProtection,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferencePublishFailure> {
        let candidate = successor.snapshot().clone();
        let mut held = Some(successor);
        self.replace_expected_permitted(expected, &candidate, &mut held, |held| held, None, None)
            .map_err(|loss| ProductBranchReferencePublishFailure {
                denial: loss.denial,
                observed_head: loss.observed_head,
                successor_protection: held
                    .take()
                    .expect("a losing test CAS retains its successor"),
            })
    }
}
