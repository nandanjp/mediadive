# syntax=docker/dockerfile:1
#
# Built from the repository root, because web depends on the generated contract
# in packages/api-types:
#   docker build -f web/Dockerfile -t mediadive-web .

FROM node:24-alpine AS base
ENV PNPM_HOME=/pnpm \
    PATH=/pnpm:$PATH \
    COREPACK_ENABLE_DOWNLOAD_PROMPT=0 \
    NEXT_TELEMETRY_DISABLED=1
RUN corepack enable pnpm
WORKDIR /build

# Manifests only, so the install layer survives every change that is not a
# dependency change.
FROM base AS deps
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY web/package.json web/
COPY packages/api-types/package.json packages/api-types/
RUN pnpm install --frozen-lockfile

FROM base AS builder
COPY --from=deps /build/node_modules node_modules
COPY --from=deps /build/web/node_modules web/node_modules
COPY --from=deps /build/packages/api-types/node_modules packages/api-types/node_modules
COPY . .
# next build runs here with no runtime configuration, so env validation is
# skipped — deliberately, and only here. See web/src/env.ts.
ENV SKIP_ENV_VALIDATION=1
RUN pnpm -C web run build

FROM node:24-alpine AS runtime
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    PORT=3000 \
    HOSTNAME=0.0.0.0
WORKDIR /app
USER node
# standalone output mirrors the workspace layout and bundles its own
# node_modules, so nothing is installed in this stage.
COPY --from=builder --chown=node:node /build/web/.next/standalone ./
COPY --from=builder --chown=node:node /build/web/.next/static ./web/.next/static
COPY --from=builder --chown=node:node /build/web/public ./web/public
EXPOSE 3000
CMD ["node", "web/server.js"]
