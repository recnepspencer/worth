use worth_relational::facade::runtime::{
    InvariantExecutionPoint, InvariantFailureEffect, InvariantRegistration, InvariantRule,
};

fn main() {
    let _ = InvariantRegistration::for_rule(
        InvariantRule::MaxMergedIntents(1),
        InvariantExecutionPoint::CommitBoundary,
        InvariantFailureEffect::BlockCommit,
    );
}
