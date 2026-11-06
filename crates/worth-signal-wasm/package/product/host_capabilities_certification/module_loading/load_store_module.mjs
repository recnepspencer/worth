import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { stripTypeScriptTypes } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.join(moduleDir, "..", "..", "..");
const reactDir = path.join(packageDir, "..", "react");

export async function loadStoreModule() {
  const tempDir = await mkdtemp(path.join(tmpdir(), "worth-signal-react-store-"));
  const sourceFiles = [
    ["model.ts", "model.js"],
    ["signal_snapshot_equality.ts", "signal_snapshot_equality.js"],
    ["store.ts", "store.js"],
  ];
  try {
    for (const [sourceName, outputName] of sourceFiles) {
      const sourcePath = path.join(reactDir, sourceName);
      const source = await readFile(sourcePath, "utf8");
      const transformed = stripTypeScriptTypes(source, { mode: "transform" });
      await writeFile(path.join(tempDir, outputName), transformed, "utf8");
    }
    const moduleUrl = new URL(
      `file:///${path.join(tempDir, "store.js").replace(/\\/g, "/")}`,
    );
    const loaded = await import(moduleUrl.href);
    return {
      ...loaded,
      cleanup: () => rm(tempDir, { recursive: true, force: true }),
    };
  } catch (error) {
    await rm(tempDir, { recursive: true, force: true });
    throw error;
  }
}
