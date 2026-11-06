import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { SyntaxKind } from "typescript/unstable/ast";
import {
  isClassDeclaration,
  isFunctionDeclaration,
  isInterfaceDeclaration,
  isTypeAliasDeclaration,
  isVariableDeclaration,
} from "typescript/unstable/ast/is";
import { API, SymbolFlags } from "typescript/unstable/sync";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
export const defaultCrateDir = path.resolve(moduleDir, "..", "..", "..");

function normalizePath(filePath) {
  return filePath.replaceAll("\\", "/");
}

function sourceDeclarationPath(publishedTypesPath) {
  const relativePath = publishedTypesPath.replace(/^\.\//u, "");
  return relativePath.startsWith("react/") ? relativePath : `package/${relativePath}`;
}

function publishedEntrypointId(exportPath) {
  return exportPath === "." ? "root" : exportPath.replace(/^\.\//u, "");
}

function readEntrypointDefinitions(crateDir) {
  const publishedManifest = JSON.parse(readFileSync(path.join(crateDir, "pkg", "package.json"), "utf8"));
  return Object.entries(publishedManifest.exports).flatMap(([exportPath, definition]) => {
    // Asset-only exports (for example ./wasm and ./worker) are string paths with
    // no TypeScript declaration surface.
    if (typeof definition === "string") {
      return [];
    }
    if (!definition || typeof definition !== "object" || typeof definition.types !== "string") {
      throw new Error(`Published entrypoint ${exportPath} lacks a declaration path`);
    }
    return [{
      declarationPath: sourceDeclarationPath(definition.types),
      id: publishedEntrypointId(exportPath),
      publishedDeclarationPath: `pkg/${definition.types.replace(/^\.\//u, "")}`,
    }];
  });
}

function assertPublishedDeclarationsSynchronized(crateDir, entrypoints) {
  for (const entrypoint of entrypoints) {
    const sourceText = readFileSync(path.join(crateDir, entrypoint.declarationPath), "utf8");
    const publishedText = readFileSync(path.join(crateDir, entrypoint.publishedDeclarationPath), "utf8");
    if (sourceText !== publishedText) {
      throw new Error(
        `Published declaration ${entrypoint.publishedDeclarationPath} differs from ${entrypoint.declarationPath}`,
      );
    }
  }
}

function declarationKind(declaration) {
  if (isClassDeclaration(declaration)) return "class";
  if (isFunctionDeclaration(declaration)) return "function";
  if (isInterfaceDeclaration(declaration)) return "interface";
  if (isTypeAliasDeclaration(declaration)) return "type";
  if (isVariableDeclaration(declaration)) return "value";
  return SyntaxKind[declaration.kind];
}

function resolveExportSymbol(checker, symbol) {
  return symbol.flags & SymbolFlags.Alias
    ? checker.getAliasedSymbol(symbol)
    : symbol;
}

// The TypeScript 7 printer emits declarations without their leading comments,
// so digests track the declared contract rather than its JSDoc prose.
function normalizedDeclarationText(emitter, node) {
  return emitter.printNode(node).replaceAll(/\s+/gu, " ").trim();
}

function namedMembers(declaration) {
  if (!isInterfaceDeclaration(declaration) && !isClassDeclaration(declaration)) return [];
  return declaration.members
    .map((member) => member.name?.getText(declaration.getSourceFile()) ?? null)
    .filter((name) => name !== null)
    .sort();
}

// The TypeScript 7 API loads projects, not loose root files: describe the
// published entrypoints as a throwaway project.
function withEntrypointProject(crateDir, entrypoints, collect) {
  const configDir = mkdtempSync(path.join(tmpdir(), "worth-signals-public-surface-"));
  const configPath = path.join(configDir, "tsconfig.json");
  writeFileSync(configPath, JSON.stringify({
    compilerOptions: {
      module: "nodenext",
      moduleResolution: "nodenext",
      noEmit: true,
      skipLibCheck: true,
      target: "esnext",
      types: [],
    },
    files: entrypoints.map((entrypoint) => path.join(crateDir, entrypoint.declarationPath)),
  }));
  const api = new API({ cwd: crateDir });
  try {
    const project = api.updateSnapshot({ openProjects: [configPath] }).getProject(configPath);
    if (!project) throw new Error("TypeScript did not load the public declaration project");
    return collect(project);
  } finally {
    api.close();
    rmSync(configDir, { force: true, recursive: true });
  }
}

function assertDeclarationsTypecheck(crateDir, program) {
  const diagnostics = [
    ...program.getConfigFileParsingDiagnostics(),
    ...program.getSyntacticDiagnostics(),
    ...program.getGlobalDiagnostics(),
    ...program.getSemanticDiagnostics(),
  ];
  if (diagnostics.length > 0) {
    const message = diagnostics
      .map((diagnostic) => `${diagnostic.fileName ? normalizePath(path.relative(crateDir, diagnostic.fileName)) : "(global)"}: TS${diagnostic.code} ${diagnostic.text}`)
      .join("\n");
    throw new Error(`Public declarations do not typecheck:\n${message}`);
  }
}

function collectDeclarationFileInventory({ crateDir, emitter, program }) {
  return program.getSourceFileNames()
    .map((fileName) => ({ fileName, source: normalizePath(path.relative(crateDir, fileName)) }))
    .filter(({ source }) =>
      (source.startsWith("package/") || source.startsWith("react/"))
      && source.endsWith(".d.ts"),
    )
    .map(({ fileName, source }) => ({
      source,
      signature: normalizedDeclarationText(emitter, program.getSourceFile(fileName)),
    }))
    .sort((left, right) => left.source.localeCompare(right.source));
}

function collectEntrypointSurfaces({ checker, crateDir, emitter, entrypoint, program }) {
  const sourcePath = path.join(crateDir, entrypoint.declarationPath);
  const source = program.getSourceFile(sourcePath);
  if (!source) throw new Error(`Missing declaration entrypoint: ${entrypoint.declarationPath}`);
  const moduleSymbol = checker.getSymbolAtLocation(source);
  if (!moduleSymbol) throw new Error(`Declaration entrypoint is not a module: ${entrypoint.declarationPath}`);

  return checker.getExportsOfModule(moduleSymbol).map((exportedSymbol) => {
    const resolvedSymbol = resolveExportSymbol(checker, exportedSymbol);
    const declaration = resolvedSymbol.declarations[0]?.resolve();
    if (!declaration) throw new Error(`Export ${exportedSymbol.name} has no declaration`);
    const source = normalizePath(path.relative(crateDir, declaration.getSourceFile().fileName));
    const signature = normalizedDeclarationText(emitter, declaration);
    const members = namedMembers(declaration);
    return {
      entrypoint: entrypoint.id,
      exportName: exportedSymbol.name,
      kind: declarationKind(declaration),
      memberCount: members.length,
      members,
      signature,
      source,
    };
  });
}

export function collectPublicContractInventory(crateDir = defaultCrateDir) {
  const entrypointDefinitions = readEntrypointDefinitions(crateDir);
  assertPublishedDeclarationsSynchronized(crateDir, entrypointDefinitions);
  return withEntrypointProject(crateDir, entrypointDefinitions, ({ checker, emitter, program }) => {
    assertDeclarationsTypecheck(crateDir, program);
    const surfaces = entrypointDefinitions
      .flatMap((entrypoint) => collectEntrypointSurfaces({ checker, crateDir, emitter, entrypoint, program }))
      .sort((left, right) =>
        left.entrypoint.localeCompare(right.entrypoint)
        || left.exportName.localeCompare(right.exportName),
      );
    return {
      declarationFiles: collectDeclarationFileInventory({ crateDir, emitter, program }),
      entrypoints: entrypointDefinitions.map((entrypoint) => entrypoint.id),
      surfaces,
    };
  });
}

function matchesRule(surface, rule) {
  return rule.entrypoints.includes(surface.entrypoint)
    && rule.sourcePrefixes.some((prefix) => surface.source.startsWith(prefix));
}

export function assignSurfacesToCoverageGroups(surfaces, policy) {
  const assignments = new Map(policy.groups.map((group) => [group.id, []]));
  const unassigned = [];
  const ambiguous = [];
  for (const surface of surfaces) {
    const matchingGroups = policy.groups.filter((group) => matchesRule(surface, group));
    if (matchingGroups.length === 0) {
      unassigned.push(surface);
    } else if (matchingGroups.length > 1) {
      ambiguous.push({ groups: matchingGroups.map((group) => group.id), surface });
    } else {
      assignments.get(matchingGroups[0].id).push(surface);
    }
  }
  return { ambiguous, assignments, unassigned };
}

export function assignDeclarationFilesToCoverageGroups(declarationFiles, policy) {
  const assignments = new Map(policy.groups.map((group) => [group.id, []]));
  const unassigned = [];
  const ambiguous = [];
  for (const declarationFile of declarationFiles) {
    const matchingGroups = policy.groups.filter((group) =>
      group.filePrefixes.some((prefix) => declarationFile.source.startsWith(prefix)),
    );
    if (matchingGroups.length === 0) {
      unassigned.push(declarationFile);
    } else if (matchingGroups.length > 1) {
      ambiguous.push({ file: declarationFile, groups: matchingGroups.map((group) => group.id) });
    } else {
      assignments.get(matchingGroups[0].id).push(declarationFile);
    }
  }
  return { ambiguous, assignments, unassigned };
}

function stableSurfaceLine(surface) {
  return [
    surface.entrypoint,
    surface.exportName,
    surface.kind,
    surface.source,
    surface.signature,
  ].join("\u001f");
}

export function summarizeSurfaceCoverageGroup(surfaces) {
  const hash = createHash("sha256");
  for (const surface of surfaces) hash.update(`${stableSurfaceLine(surface)}\n`);
  return {
    declarationCount: surfaces.length,
    directMemberCount: surfaces.reduce((total, surface) => total + surface.memberCount, 0),
    signatureDigest: hash.digest("hex"),
  };
}

function summarizeDeclarationFiles(declarationFiles) {
  const hash = createHash("sha256");
  for (const declarationFile of declarationFiles) {
    hash.update(`${declarationFile.source}\u001f${declarationFile.signature}\n`);
  }
  return {
    declarationFileCount: declarationFiles.length,
    declarationFileDigest: hash.digest("hex"),
  };
}

export function summarizeCoveragePolicy(contractInventory, policy) {
  const surfaceCoverage = assignSurfacesToCoverageGroups(contractInventory.surfaces, policy);
  const fileCoverage = assignDeclarationFilesToCoverageGroups(contractInventory.declarationFiles, policy);
  const groups = policy.groups.map((group) => ({
    id: group.id,
    ...summarizeSurfaceCoverageGroup(surfaceCoverage.assignments.get(group.id)),
    ...summarizeDeclarationFiles(fileCoverage.assignments.get(group.id)),
  }));
  return { fileCoverage, groups, surfaceCoverage };
}

export function publicSurfaceName(surface) {
  return `${surface.entrypoint}:${surface.exportName}`;
}
