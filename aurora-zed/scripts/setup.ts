import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const zedDir = resolve(scriptDir, "..");
const repoRoot = resolve(zedDir, "..");
const extensionTomlPath = resolve(zedDir, "extension.toml");
const grammarDir = resolve(repoRoot, "aurora-tree-sitter");
const localGrammarCheckout = resolve(zedDir, "grammars", "aurora");
const localGrammarRepoUrl = pathToFileURL(repoRoot).href;

const mode = parseMode(Bun.argv.slice(2));
const manifest = mode === "dev" ? devManifest() : prodManifest();

if (mode === "dev") {
  prepareDevEnvironment();
} else {
  prepareProdEnvironment();
}

await Bun.write(extensionTomlPath, manifest);

if (mode === "dev") {
  console.log(`aurora-zed uses local grammar checkout: ${localGrammarCheckout}`);
  console.log("Run `moon run aurora-zed:setup -- --prod` before committing.");
} else {
  console.log("aurora-zed uses remote grammar: https://github.com/JustKira/aurora-orm.git#main");
}

function parseMode(args: string[]): "dev" | "prod" {
  const hasDev = args.includes("--dev");
  const hasProd = args.includes("--prod");

  if (hasDev === hasProd) {
    usage();
    process.exit(1);
  }

  return hasDev ? "dev" : "prod";
}

function usage(): void {
  console.error("Usage: bun aurora-zed/scripts/setup.ts (--dev | --prod)");
}

function prepareLocalGrammarCheckout(): void {
  rmSync(localGrammarCheckout, { recursive: true, force: true });
  mkdirSync(localGrammarCheckout, { recursive: true });
  run(["git", "init", "-q"], localGrammarCheckout);
  run(["git", "remote", "add", "origin", localGrammarRepoUrl], localGrammarCheckout);
  run(["git", "fetch", "--depth", "1", "origin", "HEAD"], localGrammarCheckout);
  run(["git", "checkout", "-q", "FETCH_HEAD"], localGrammarCheckout);

  const localGrammarPath = resolve(localGrammarCheckout, "aurora-tree-sitter");
  const nodeModulesPath = resolve(grammarDir, "node_modules");
  rmSync(localGrammarPath, { recursive: true, force: true });
  cpSync(grammarDir, localGrammarPath, {
    recursive: true,
    filter: (source) =>
      source !== nodeModulesPath && !source.startsWith(`${nodeModulesPath}${sep}`),
  });
  rmSync(resolve(localGrammarPath, ".git"), { recursive: true, force: true });
}

function prepareDevEnvironment(): void {
  clearZedGrammarCaches();
  prepareLocalGrammarCheckout();
}

function prepareProdEnvironment(): void {
  rmSync(resolve(zedDir, "grammars"), { recursive: true, force: true });
}

function clearZedGrammarCaches(): void {
  const cachePaths = [
    resolve(homeDir(), "Library", "Application Support", "Zed", "extensions", "work", "aurora"),
  ];

  for (const cachePath of cachePaths) {
    rmSync(cachePath, { recursive: true, force: true });
  }
}

function homeDir(): string {
  const home = process.env.HOME ?? process.env.USERPROFILE;

  if (!home) {
    throw new Error("Could not determine the home directory for Zed cache cleanup.");
  }

  return home;
}

function run(cmd: string[], cwd = repoRoot): void {
  const result = Bun.spawnSync(cmd, {
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  });

  if (!result.success) {
    throw new Error(`command failed: ${cmd.join(" ")}`);
  }
}

function devManifest(): string {
  return `id = "aurora"
name = "Aurora"
description = "Aurora schema language support for Zed"
version = "0.1.0"
schema_version = 1
authors = ["Aurora"]

[grammars.aurora]
repository = "${localGrammarRepoUrl}"
rev = "HEAD"
path = "aurora-tree-sitter"

[language_servers.aurora-lsp]
name = "Aurora LSP"
languages = ["Aurora"]
`;
}

function prodManifest(): string {
  return `id = "aurora"
name = "Aurora"
description = "Aurora schema language support for Zed"
version = "0.1.0"
schema_version = 1
authors = ["Aurora"]

# The grammar lives in this repo at aurora-tree-sitter/. The \`path\` field
# tells Zed to look in that subdirectory after cloning. The field is real
# (zed/crates/extension/src/extension_manifest.rs:310) even though it's not
# in the public docs. Update \`rev\` when the grammar changes — pin to a
# commit SHA for published versions, branch name is fine for dev.
[grammars.aurora]
repository = "https://github.com/JustKira/aurora-orm.git"
rev = "main"
path = "aurora-tree-sitter"

[language_servers.aurora-lsp]
name = "Aurora LSP"
languages = ["Aurora"]
`;
}
