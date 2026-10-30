use worth_relational::facade::commit_strategies::CommitStrategiesAuthorityFacade;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;
use worth_relational::facade::runtime::RelationalRuntime;

fn deny(mut runtime: RelationalRuntime, proposal: ValidatedRelationalProposal) {
    runtime.commit_validated_proposal(proposal);

    let mut strategies = CommitStrategiesAuthorityFacade::default();
    strategies.execute_validated_commit(&runtime, proposal);
}

fn main() {}
