import { defineConfig, defineDocs } from "fumadocs-mdx/config";
import type { LanguageInput } from "shiki";
import auroraGrammar from "./grammars/aurora.tmLanguage.json";
import surrealqlGrammar from "./grammars/surrealql.tmLanguage.json";

const auroraLanguage = auroraGrammar as unknown as LanguageInput;
const surrealqlLanguage = surrealqlGrammar as unknown as LanguageInput;

export const docs = defineDocs({
  dir: "content/docs",
  docs: {
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
});

export default defineConfig({
  mdxOptions: {
    rehypeCodeOptions: {
      themes: {
        light: "ayu-light",
        dark: "ayu-dark",
      },
      langAlias: {
        npm: "bash",
      },
      langs: [
        surrealqlLanguage,
        auroraLanguage,
        "ts",
        "tsx",
        "js",
        "json",
        "bash",
        "shell",
        "md",
        "mdx",
      ],
    },
  },
});
