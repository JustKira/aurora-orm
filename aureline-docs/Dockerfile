#### BASE STAGE ####
FROM oven/bun:1.3.11 AS base

WORKDIR /app

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends ca-certificates curl git \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://moonrepo.dev/install/moon.sh | bash

ENV PATH="/root/.moon/bin:/root/.proto/bin:$PATH"

#### SKELETON STAGE ####
FROM base AS skeleton

COPY . .
RUN moon docker scaffold docs

#### BUILD STAGE ####
FROM base AS build

COPY --from=skeleton /app/.moon/docker/workspace .
RUN moon docker setup

COPY --from=skeleton /app/.moon/docker/sources .
RUN moon run docs:build
RUN moon docker prune

#### START STAGE ####
FROM oven/bun:1.3.11 AS start

WORKDIR /app

ENV HOST=0.0.0.0
ENV PORT=3000
ENV NODE_ENV=production

COPY --from=build /app/aureline-docs/.output ./.output

EXPOSE 3000

CMD ["bun", ".output/server/index.mjs"]
