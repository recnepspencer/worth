use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::source_delta::{
    CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta, MalformedPulseSourceDelta,
    RevisionSchemaSourceDelta, StatusSchemaRecoverySourceDelta,
};

use super::{
    AwaitingPreservation, AwaitingRecovery, AwaitingReplacement, AwaitingSchemaStop,
    AwaitingStatusRecovery, GreenSuccessor, PreservedPredecessor, PreservedPredecessorEvidence,
    Published, PulseExecutableWorld, RecoveredBlue, SchemaStopped, SecondCurrent,
};

impl PulseExecutableWorld<Published<SecondCurrent>> {
    pub(crate) fn apply_green(
        self,
        delta: GreenPulseSourceDelta,
    ) -> Result<PulseExecutableWorld<AwaitingReplacement>, PulseExecutableWorldFailureReport> {
        let Published { world, stage } = self.state;
        let action = match delta.apply(&world.installation) {
            Ok(action) => action,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::SourceAction(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingReplacement {
                world,
                initial: stage,
                action,
            },
        })
    }
}

impl PulseExecutableWorld<Published<RecoveredBlue>> {
    pub(crate) fn apply_revision_schema(
        self,
        delta: RevisionSchemaSourceDelta,
    ) -> Result<PulseExecutableWorld<AwaitingSchemaStop>, PulseExecutableWorldFailureReport> {
        let Published { world, stage } = self.state;
        let action = match delta.apply(&world.installation) {
            Ok(action) => action,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::SourceAction(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingSchemaStop {
                world,
                recovered: stage,
                action,
            },
        })
    }
}

impl PulseExecutableWorld<Published<SchemaStopped>> {
    pub(crate) fn restore_status_schema(
        self,
        delta: StatusSchemaRecoverySourceDelta,
    ) -> Result<PulseExecutableWorld<AwaitingStatusRecovery>, PulseExecutableWorldFailureReport>
    {
        let Published { world, stage } = self.state;
        let action = match delta.apply(&world.installation) {
            Ok(action) => action,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::SourceAction(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingStatusRecovery {
                world,
                stopped: stage,
                action,
            },
        })
    }
}

impl PulseExecutableWorld<Published<GreenSuccessor>> {
    pub(crate) fn apply_malformed(
        self,
        delta: MalformedPulseSourceDelta,
    ) -> Result<PulseExecutableWorld<AwaitingPreservation>, PulseExecutableWorldFailureReport> {
        let Published { world, stage } = self.state;
        let action = match delta.apply(&world.installation) {
            Ok(action) => action,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::SourceAction(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingPreservation {
                world,
                green: stage,
                action,
            },
        })
    }
}

impl PulseExecutableWorld<PreservedPredecessor> {
    pub(crate) fn restore_canonical(
        self,
        delta: CanonicalBlueRecoverySourceDelta,
    ) -> Result<PulseExecutableWorld<AwaitingRecovery>, PulseExecutableWorldFailureReport> {
        let PreservedPredecessor {
            world,
            green,
            evidence,
        } = self.state;
        let action = match delta.apply(&world.installation) {
            Ok(action) => action,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::SourceAction(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingRecovery {
                world,
                preserved: PreservedPredecessorEvidence { green, evidence },
                action,
            },
        })
    }
}
