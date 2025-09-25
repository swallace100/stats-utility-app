# ---------- deps (backend manifests + install) ----------
FROM node:20.14-alpine AS deps
WORKDIR /app

# Copy manifests for both backend and contracts so npm can resolve file: dep
# (build context must be repo root)
COPY package*.json ./
COPY apps/backend/package*.json apps/backend/
COPY apps/packages/contracts/package*.json apps/packages/contracts/

# Install backend deps (records the file: dep)
WORKDIR /app/apps/backend
RUN npm ci --no-audit --no-fund

# ---------- build contracts (to have dist for runtime) ----------
FROM node:20.14-alpine AS contracts
WORKDIR /app
# bring in the contracts sources from the build context
COPY apps/packages/contracts apps/packages/contracts
WORKDIR /app/apps/packages/contracts
# if there's no package-lock.json here, use npm install (not ci)
RUN npm install --no-audit --no-fund
RUN npm run build  # emits dist/ (+ .d.ts if configured)

# ---------- build backend ----------
FROM node:20.14-alpine AS build
WORKDIR /app

# Use the node_modules we installed for backend
COPY --from=deps /app/apps/backend/node_modules apps/backend/node_modules

# Copy backend sources
COPY apps/backend/package*.json apps/backend/
COPY apps/backend/tsconfig*.json apps/backend/
COPY apps/backend/src apps/backend/src

# Also copy contracts *source* so TS can compile against it via path mapping
COPY apps/packages/contracts/src apps/packages/contracts/src

# Switch into backend
WORKDIR /app/apps/backend

# Create a local tsconfig that maps @your-scope/contracts -> ../packages/contracts/src
# This avoids needing node_modules during compile
RUN node -e '\
    const fs = require("fs"); \
    const base = JSON.parse(fs.readFileSync("tsconfig.build.json","utf8")); \
    base.compilerOptions = base.compilerOptions || {}; \
    base.compilerOptions.baseUrl = base.compilerOptions.baseUrl || "."; \
    base.compilerOptions.paths = Object.assign({}, base.compilerOptions.paths, { \
    "@your-scope/contracts": ["../packages/contracts/src"], \
    "@your-scope/contracts/*": ["../packages/contracts/src/*"] \
    }); \
    fs.writeFileSync("tsconfig.build.local.json", JSON.stringify(base, null, 2));'

# Compile backend TS -> dist using the local tsconfig with paths
RUN npx tsc -p tsconfig.build.local.json

# ---------- runtime ----------
FROM node:20.14-alpine
WORKDIR /app
ENV NODE_ENV=production

# non-root user
RUN adduser -D app && chown -R app:app /app
USER app

# Install only production deps for backend
COPY apps/backend/package*.json ./
RUN npm ci --omit=dev --no-audit --no-fund

# Bring compiled backend
COPY --from=build /app/apps/backend/dist ./dist

# Ship the built contracts package into runtime node_modules (so require("@your-scope/contracts") works)
RUN mkdir -p node_modules/@your-scope/contracts
COPY --from=contracts /app/apps/packages/contracts/dist node_modules/@your-scope/contracts/dist
COPY apps/packages/contracts/package.json node_modules/@your-scope/contracts/package.json

EXPOSE 8080
CMD ["node","dist/server.js"]
