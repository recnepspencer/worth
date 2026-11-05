/** A readback basis includes host refreshes, which can run beside input writes. */
export function createHostRefreshCurrentness() {
  let epoch = 0;
  const pending = new Set();
  return {
    epoch: () => epoch,
    isCurrent: basis => basis === epoch && pending.size === 0,
    begin() {
      epoch += 1;
      let resolve;
      const completion = new Promise(done => { resolve = done; });
      pending.add(completion);
      return () => {
        pending.delete(completion);
        resolve();
      };
    },
    async settle() {
      while (pending.size > 0) await Promise.all([...pending]);
    },
  };
}
