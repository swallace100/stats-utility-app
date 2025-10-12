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
# remove the file: dependency to avoid npm trying to pack/prepare it here
RUN node -e "const fs=require('fs');const p=JSON.parse(fs.readFileSync('package.json','utf8')); if(p.dependencies && p.dependencies['@your-scope/contracts']){ delete p.dependencies['@your-scope/contracts']; fs.writeFileSync('package.json', JSON.stringify(p,null,2)); }"
# use npm install (not ci) since lock now differs; ignore scripts for speed/stability
RUN npm install --no-audit --no-fund --ignore-scripts
# (produces a package-lock.json we will reuse later)

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
    "rootDir": "src",
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

# Reuse the mutated package.json from deps
COPY --from=deps /app/apps/backend/package.json ./
# Bring the built contracts tarball
COPY --from=contracts /tmp/contracts.tgz /tmp/contracts.tgz

# Install production deps (no dev), then add contracts tarball
RUN npm install --omit=dev --no-audit --no-fund \
  && npm install --omit=dev --no-audit --no-fund /tmp/contracts.tgz \
  && rm -f /tmp/contracts.tgz

# 👉 Ensure curl is available for healthchecks/smoke
RUN apk add --no-cache curl

# Create a writable data dir under /app
RUN mkdir -p /app/data

# Drop privileges AFTER installs so /app is owned correctly
RUN adduser -D app && chown -R app:app /app
USER app

# Bring compiled backend
COPY --from=build /app/apps/backend/dist ./dist

EXPOSE 8080
CMD ["node","dist/server.js"]
