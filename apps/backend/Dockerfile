# 1) Install deps (cached)
FROM node:20.14-alpine AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci --no-audit --no-fund

# 2) Build TypeScript to dist/
FROM node:20.14-alpine AS build
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY package*.json ./
COPY tsconfig.json ./
COPY src ./src
# If you have other build assets, copy them too (e.g., prisma/schema.prisma)
RUN npx tsc -p tsconfig.build.json

# 3) Runtime with only production deps + compiled JS
FROM node:20.14-alpine
WORKDIR /app
ENV NODE_ENV=production

# Run as non-root
RUN adduser -D app && chown -R app:app /app
USER app

# Install only prod deps for smaller image
COPY package*.json ./
RUN npm ci --omit=dev --no-audit --no-fund

# Bring in compiled output
COPY --from=build /app/dist ./dist

EXPOSE 8080
# Make sure your entry is dist/server.js (compiled from src/server.ts)
CMD ["node","dist/server.js"]
