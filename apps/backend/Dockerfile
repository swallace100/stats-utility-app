# ---------- deps layer ----------
FROM node:20.14-alpine AS deps
WORKDIR /app

# Copy manifests for both backend and contracts so npm can resolve file: dep
COPY package*.json ./
COPY apps/backend/package*.json apps/backend/
COPY apps/packages/contracts/package*.json apps/packages/contracts/

# Install *backend* deps; npm will resolve the file: dependency path
WORKDIR /app/apps/backend
RUN npm ci --no-audit --no-fund

# ---------- build contracts ----------
FROM node:20.14-alpine AS contracts
WORKDIR /app
COPY apps/packages/contracts apps/packages/contracts
WORKDIR /app/apps/packages/contracts
RUN npm ci --no-audit --no-fund
RUN npm run build

# ---------- build backend ----------
FROM node:20.14-alpine AS build
WORKDIR /app

# Bring node_modules installed for backend (with a placeholder for contracts)
COPY --from=deps /app/apps/backend/node_modules apps/backend/node_modules
COPY apps/backend/package*.json apps/backend/
COPY apps/backend/tsconfig*.json apps/backend/
COPY apps/backend/src apps/backend/src

# Provide the built contracts into node_modules (npm file: will be satisfied at runtime)
# We create the folder and place the built dist + package.json
RUN mkdir -p /app/apps/backend/node_modules/@your-scope/contracts
COPY --from=contracts /app/apps/packages/contracts/dist /app/apps/backend/node_modules/@your-scope/contracts/dist
# also copy the contracts package.json so Node can resolve "main":"dist/index.js"
COPY apps/packages/contracts/package.json /app/apps/backend/node_modules/@your-scope/contracts/package.json

# Compile backend TS -> dist
WORKDIR /app/apps/backend
RUN npx tsc -p tsconfig.build.json

# ---------- runtime ----------
FROM node:20.14-alpine
WORKDIR /app
ENV NODE_ENV=production

# non-root user
RUN adduser -D app && chown -R app:app /app
USER app

# Install only production deps for backend (no need to rebuild contracts here)
COPY apps/backend/package*.json ./
RUN npm ci --omit=dev --no-audit --no-fund

# Drop in compiled backend
COPY --from=build /app/apps/backend/dist ./dist

# Also drop in the built contracts (so require('@your-scope/contracts') resolves)
RUN mkdir -p node_modules/@your-scope/contracts
COPY --from=contracts /app/apps/packages/contracts/dist node_modules/@your-scope/contracts/dist
COPY apps/packages/contracts/package.json node_modules/@your-scope/contracts/package.json

EXPOSE 8080
CMD ["node","dist/server.js"]
