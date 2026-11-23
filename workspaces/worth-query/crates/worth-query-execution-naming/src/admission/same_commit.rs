//! Issues the reference to a name an authored program's own output will be
//! born with. A reference exists only for an output the program names.

use worth_query_declaration::facade::application_naming::WorthQuerySameCommitName;
use worth_query_declaration::persistent_name_admission_authority::issue_same_commit_name;

use crate::operation_outputs::OperationNamingProgram;

/// The same-commit name of the output `program` names at `target`, issued
/// under the program's own identity; `None` when it names no such output.
///
/// `K` is the kind of the created-output handle the caller holds for
/// `target`.
///
/// Complexity: O(E) for E named outputs.
pub fn same_commit_name_of<K, T: PartialEq>(
    program: &OperationNamingProgram<T>,
    target: &T,
) -> Option<WorthQuerySameCommitName<K>> {
    program
        .named_outputs()
        .position(|output| output.target == *target)
        .map(|output| issue_same_commit_name(program.candidate(), output))
}
