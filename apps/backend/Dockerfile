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

# ---------- build contracts (to have dist for runtime & types for TS) ----------
FROM node:20.14-alpine AS contracts
WORKDIR /app
COPY apps/packages/contracts apps/packages/contracts
WORKDIR /app/apps/packages/contracts
RUN npm install --no-audit --no-fund
RUN npm run build  # emits dist/ (+ .d.ts if configured)
# create a tarball we can install in runtime
RUN npm pack --silent && mv *.tgz /tmp/contracts.tgz

# ---------- build backend ----------
FROM node:20.14-alpine AS build
WORKDIR /app

# Use the node_modules we installed for backend
COPY --from=deps /app/apps/backend/node_modules apps/backend/node_modules

# Copy backend sources
COPY apps/backend/package*.json apps/backend/
COPY apps/backend/tsconfig*.json apps/backend/
COPY apps/backend/src apps/backend/src

# Copy ONLY contracts *dist* (types) for compile-time path mapping
COPY --from=contracts /app/apps/packages/contracts/dist apps/packages/contracts/dist

# Switch into backend
WORKDIR /app/apps/backend

# Create a local tsconfig that extends your build config and maps the package to dist types
RUN cat > tsconfig.build.local.json <<'EOF'
{
  "extends": "./tsconfig.build.json",
  "compilerOptions": {
    "rootDir": ".",
    "baseUrl": ".",
    "paths": {
      "@your-scope/contracts": ["../packages/contracts/dist"],
      "@your-scope/contracts/*": ["../packages/contracts/dist/*"]
    }
  }
}
EOF

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

# Install the built contracts package tarball
COPY --from=contracts /tmp/contracts.tgz /tmp/contracts.tgz
RUN npm install --omit=dev --no-audit --no-fund /tmp/contracts.tgz

# Bring compiled backend
COPY --from=build /app/apps/backend/dist ./dist

EXPOSE 8080
CMD ["node","dist/server.js"]
