interface GearCodeSources {
  compositionSection: string;
  gearScenario: string;
}

interface SourceExcerpt {
  end: string;
  file: string;
  source: string;
  start: string;
}

export function buildGearCodeEvidence(sources: GearCodeSources) {
  return [
    excerpt({
      file: "CompositionSection.tsx",
      source: sources.compositionSection,
      start: "  const commitBranchAspect = (",
      end: "\n\n  const forkBranches =",
    }),
    excerpt({
      file: "gear_scenario.ts",
      source: sources.gearScenario,
      start: "      const branchId = role === \"main\" ? main.id : activeDesignBranchId;",
      end: "\n      const aspectNames =",
    }),
    excerpt({
      file: "gear_scenario.ts",
      source: sources.gearScenario,
      start: "  async function commitOperations(",
      end: "\n\n  /** Commit projections",
    }),
    excerpt({
      file: "gear_scenario.ts",
      source: sources.gearScenario,
      start: "    async selectHistoryCommit(",
      end: "\n    async terminate()",
    }),
  ].join("\n\n");
}

function excerpt({ end, file, source, start }: SourceExcerpt) {
  const normalizedSource = source.replace(/\r\n?/g, "\n");
  const startIndex = normalizedSource.indexOf(start);
  const endIndex = normalizedSource.indexOf(end, startIndex);
  if (startIndex < 0 || endIndex < 0) {
    throw new Error(`Demo 6 code evidence is missing its ${file} production excerpt.`);
  }
  return `// ${file} — production source\n${dedent(normalizedSource.slice(startIndex, endIndex))}`;
}

function dedent(source: string) {
  const lines = source.split("\n");
  const indentation = Math.min(
    ...lines.filter((line) => line.trim()).map((line) => line.match(/^\s*/)?.[0].length ?? 0),
  );
  return lines.map((line) => line.slice(indentation)).join("\n");
}
