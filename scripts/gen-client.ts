import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";
import { renderVisitor as renderJavaScriptVisitor } from "@codama/renderers-js";
import * as fs from "fs";
import * as path from "path";

const args = process.argv.slice(2);

if (args.length < 3) {
  console.error(
    "Usage: npx tsx scripts/gen-client.ts <idl-path> <rust-output-dir> <ts-output-dir>"
  );
  process.exit(1);
}

const [idlPath, rustOutputDir, tsOutputDir] = args;

const idlContent = fs.readFileSync(idlPath, "utf-8");
const idl = JSON.parse(idlContent);

const codama = createFromRoot(rootNodeFromAnchor(idl));

console.log(`Generating Rust client to ${rustOutputDir}...`);
codama.accept(renderRustVisitor(path.resolve(rustOutputDir)));

console.log(`Generating TypeScript client to ${tsOutputDir}...`);
codama.accept(renderJavaScriptVisitor(path.resolve(tsOutputDir)));

console.log("Done!");
