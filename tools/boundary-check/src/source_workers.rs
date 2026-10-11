//! Independent file work, joined in its original input order.

pub(crate) fn map<T: Sync, R: Send>(
    inputs: &[T],
    width: usize,
    operation: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    let width = width.max(1).min(inputs.len().max(1));
    if width == 1 {
        return inputs.iter().map(operation).collect();
    }
    std::thread::scope(|scope| {
        let operation = &operation;
        let workers: Vec<_> = inputs
            .chunks(inputs.len().div_ceil(width))
            .map(|chunk| scope.spawn(move || chunk.iter().map(operation).collect::<Vec<_>>()))
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| match worker.join() {
                Ok(output) => output,
                Err(panic) => std::panic::resume_unwind(panic),
            })
            .collect()
    })
}
