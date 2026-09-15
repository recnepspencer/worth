function createResourceLineEpoch() {
  let version = 0;
  const lineBackings = new Set();
  return Object.freeze({
    register(lineBacking) {
      lineBackings.add(lineBacking);
    },
    unregister(lineBacking) {
      lineBackings.delete(lineBacking);
    },
    captureAll() {
      for (const lineBacking of lineBackings) {
        lineBacking.captureCurrentSnapshot();
      }
    },
    version() {
      return version;
    },
    // Retires every current materialization: each backing rematerializes on
    // its next access. This is identity/namespace turnover, not freshness;
    // freshness invalidation is `invalidateLine` per backing.
    advanceVersion() {
      version += 1;
      return version;
    },
    // Every registered (not yet released) line backing, in registration
    // order, snapshotted so a sweep is unaffected by lines it releases or
    // materializes while running. Cost: O(registered lines).
    lineBackings() {
      return Object.freeze([...lineBackings]);
    },
  });
}

export { createResourceLineEpoch };
