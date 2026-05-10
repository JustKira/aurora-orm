import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const zedDir = resolve(scriptDir, "..");
const repoRoot = resolve(zedDir, "..");
const extensionTomlPath = resolve(zedDir, "extension.toml");
const grammarDir = resolve(repoRoot, "aurora-tree-sitter");
const localGrammarRepo = resolve(zedDir, ".local-grammar", "aurora");
const localGrammarRepoUrl = pathToFileURL(localGrammarRepo).href;

const mode = parseMode(Bun.argv.slice(2));
const manifest = mode === "dev" ? devManifest(await prepareDevEnvironment()) : prodManifest();

await Bun.write(extensionTomlPath, manifest);

if (mode === "dev") {
  console.log(`aurora-zed uses local grammar: ${localGrammarRepoUrl}`);
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

async function prepareLocalGrammarRepo(): Promise<string> {
  rmSync(localGrammarRepo, { recursive: true, force: true });
  mkdirSync(localGrammarRepo, { recursive: true });
  cpSync(grammarDir, localGrammarRepo, { recursive: true });
  rmSync(resolve(localGrammarRepo, ".git"), { recursive: true, force: true });
  run(["git", "init", "-q"], localGrammarRepo);
  run(["git", "add", "."], localGrammarRepo);
  run(
    [
      "git",
      "-c",
      "user.name=Aurora Zed Dev",
      "-c",
      "user.email=aurora-zed-dev@example.invalid",
      "commit",
      "-q",
      "-m",
      "local aurora grammar snapshot",
    ],
    localGrammarRepo,
  );

  return commandOutput(["git", "rev-parse", "HEAD"], localGrammarRepo);
}

async function prepareDevEnvironment(): Promise<string> {
  clearZedGrammarCaches();
  return prepareLocalGrammarRepo();
}

function clearZedGrammarCaches(): void {
  const cachePaths = [
    resolve(zedDir, "grammars"),
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

function commandOutput(cmd: string[], cwd = repoRoot): string {
  const result = Bun.spawnSync(cmd, {
    cwd,
    stdout: "pipe",
    stderr: "inherit",
  });

  if (!result.success) {
    throw new Error(`command failed: ${cmd.join(" ")}`);
  }

  return result.stdout.toString().trim();
}

function devManifest(rev: string): string {
  return `id = "aurora"
name = "Aurora"
description = "Aurora schema language support for Zed"
version = "0.1.0"
schema_version = 1
authors = ["Aurora"]

[grammars.aurora]
repository = "${localGrammarRepoUrl}"
rev = "${rev}"

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
