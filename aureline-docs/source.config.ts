import { defineConfig, defineDocs } from "fumadocs-mdx/config";
import type { LanguageInput } from "shiki";
import aurelineGrammar from "./grammars/aureline.tmLanguage.json";
import surrealqlGrammar from "./grammars/surrealql.tmLanguage.json";

const aurelineLanguage = aurelineGrammar as unknown as LanguageInput;
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
        aurelineLanguage,
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
