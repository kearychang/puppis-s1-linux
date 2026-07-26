import { readFileSync } from "node:fs";

const packageJson = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
const cargo = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const tauri = JSON.parse(readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
const cargoVersion = cargo.match(/^version = "([^"]+)"$/m)?.[1];
const versions = new Map([
  ["npm", packageJson.version],
  ["Rust workspace", cargoVersion],
  ["Tauri", tauri.version],
]);
const expected = packageJson.version;
const mismatches = [...versions].filter(([, version]) => version !== expected);
if (mismatches.length) {
  for (const [source, version] of mismatches) console.error(`${source} version ${version ?? "missing"} does not match ${expected}`);
  process.exit(1);
}
for (const lockfile of ["../Cargo.lock", "../package-lock.json"]) readFileSync(new URL(lockfile, import.meta.url));
console.log(`Version ${expected} is shared by npm, Rust, and Tauri; both lockfiles are present.`);
