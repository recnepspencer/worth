//! Each declared compiler obligation needs a distinct actual error.
pub(super) fn covers_all(candidates: &[Vec<usize>], errors: usize) -> bool {
    if candidates.len() != errors {
        return false;
    }
    let mut owners = vec![None; errors];
    (0..candidates.len()).all(|requirement| {
        assign(
            requirement,
            candidates,
            &mut owners,
            &mut vec![false; errors],
        )
    })
}

fn assign(
    requirement: usize,
    candidates: &[Vec<usize>],
    owners: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for &error in &candidates[requirement] {
        if visited[error] {
            continue;
        }
        visited[error] = true;
        if owners[error].is_none_or(|prior| assign(prior, candidates, owners, visited)) {
            owners[error] = Some(requirement);
            return true;
        }
    }
    false
}
