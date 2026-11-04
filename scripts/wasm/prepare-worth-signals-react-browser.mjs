import { execFile, spawn } from "node:child_process";
import { cp, mkdir, mkdtemp, writeFile } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const exec = promisify(execFile);
const root = fileURLToPath(new URL("../../", import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).map(arg => arg.replace(/^--/, "").split("=")));
const react = args.react ?? "19.2.5";
if (!["18.3.1", "19.2.5"].includes(react)) throw new Error("Use --react=18.3.1 or --react=19.2.5");
const port = Number(args.port ?? 4188);
if (!Number.isInteger(port) || port < 1024 || port > 65000) throw new Error("Invalid --port");
const pkg = path.join(root, "crates/worth-signal-wasm/pkg");
const npmCli = path.join(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js");
async function npm(npmArgs, cwd) {
  return exec(process.execPath, [npmCli, ...npmArgs], { cwd, maxBuffer: 8 * 1024 * 1024 });
}
await exec(process.execPath, ["scripts/wasm/verify-worth-signals-release-toolchain.mjs"], { cwd: root });
const world = await mkdtemp(path.join(root, "target/react-browser-"));
const packed = JSON.parse((await npm(["pack", "--json", "--pack-destination", world], pkg)).stdout);
const artifact = (Array.isArray(packed) ? packed : Object.values(packed))[0];
await cp(path.join(world, artifact.filename), path.join(world, "package.tgz"));
await cp(path.join(root, "crates/worth-signal-wasm/react/browser"), world, { recursive: true });
await writeFile(path.join(world, "package.json"), JSON.stringify({
  name: "worth-react-browser-consumer", private: true, type: "module",
  dependencies: { "worth-signals-wasm": "file:./package.tgz", react, "react-dom": react, vite: "8.0.1" },
}));
const reports = path.join(root, "target/react-browser-results");
await mkdir(reports, { recursive: true });
await writeFile(path.join(world, "vite.config.mjs"), `
import {writeFile} from "node:fs/promises";
function resultsPlugin() {
  const attach = (server, mode) => { server.middlewares.use('/__results', (request, response) => {
    if(request.method !== 'POST') { response.writeHead(405).end(); return; }
    let body = '';
    request.on('data', chunk => { body += chunk; });
    request.on('end', async () => {
      try {
        const report = JSON.parse(body);
        if (report.environment.react !== ${JSON.stringify(react)} || report.environment.mode !== mode) {
          response.writeHead(400).end('wrong test environment'); return;
        }
        const file = ${JSON.stringify(reports)} + '/' + ${JSON.stringify(react)} + '-' + mode + '-' + Date.now() + '.json';
        await writeFile(file, JSON.stringify(report, null, 2));
        response.writeHead(200).end('saved');
      } catch { response.writeHead(500).end('invalid results'); }
    });
  }); };
  return {name:'delivery-results', configureServer: server => attach(server, 'development-strict'),
    configurePreviewServer: server => attach(server, 'production')};
}
export default {plugins:[resultsPlugin()], worker:{format:'es'}, build:{target:'esnext'}, server:{host:'127.0.0.1'}, preview:{host:'127.0.0.1'}};
`);
await npm(["install", "--no-fund"], world);
const vite = path.join(world, "node_modules/vite/bin/vite.js");
console.log(`Building packed consumer with React ${react} at ${world}`);
await exec(process.execPath, [vite, "build"], { cwd: world, maxBuffer: 8 * 1024 * 1024 });
const servers = [
  spawn(process.execPath, [vite, "preview", "--port", String(port), "--strictPort"], { cwd: world, stdio: "inherit" }),
  spawn(process.execPath, [vite, "--port", String(port + 1), "--strictPort"], { cwd: world, stdio: "inherit" }),
];
console.log(`React ${react}: production http://127.0.0.1:${port}; development/StrictMode http://127.0.0.1:${port + 1}`);
console.log(`Results: ${reports}`);
process.on("SIGINT", () => { servers.forEach(server => server.kill()); process.exit(); });
process.on("SIGTERM", () => { servers.forEach(server => server.kill()); process.exit(); });
await Promise.all(servers.map(server => new Promise(resolve => server.on("exit", resolve))));
