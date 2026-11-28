use worth_relational::facade::mvcc::ValidatedRelationalProposal;
use worth_relational::facade::runtime::RelationalRuntime;

fn deny(mut runtime: RelationalRuntime, proposal: ValidatedRelationalProposal) {
    runtime.commit_validated_proposal(proposal);
}

fn main() {}
