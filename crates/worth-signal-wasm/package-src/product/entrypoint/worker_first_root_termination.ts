/** Source graphs must outlive authored callbacks that still read their handles. */
export function createWorkerFirstRootTermination(rootSession, trackedGraphs) {
  let termination = null;
  return function terminate() {
    termination ??= (async () => {
      await rootSession.settleAuthoredWork();
      const graphs = [...trackedGraphs];
      trackedGraphs.clear();
      await Promise.all(graphs.map(graph => graph.terminate()));
      await rootSession.terminate();
    })();
    return termination;
  };
}
