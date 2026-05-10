const repoRoot = import.meta.dir.replace(/\/aurora-zed\/scripts$/, "");
const zedDir = `${repoRoot}/aurora-zed`;
const extensionTomlPath = `${zedDir}/extension.toml`;
const grammarDir = `${repoRoot}/aurora-tree-sitter`;
const localGrammarRepo = `${zedDir}/.local-grammar/aurora`;

const mode = parseMode(Bun.argv.slice(2));
const manifest = mode === "dev" ? devManifest(await prepareLocalGrammarRepo()) : prodManifest();

await Bun.write(extensionTomlPath, manifest);

if (mode === "dev") {
  console.log(`aurora-zed uses local grammar: file://${localGrammarRepo}`);
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
  run(["rm", "-rf", localGrammarRepo]);
  run(["mkdir", "-p", localGrammarRepo]);
  run(["cp", "-R", `${grammarDir}/.`, localGrammarRepo]);
  run(["rm", "-rf", `${localGrammarRepo}/.git`]);
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
repository = "file://${localGrammarRepo}"
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
