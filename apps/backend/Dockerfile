FROM node:20.14-alpine AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci --no-audit --no-fund

FROM node:20.14-alpine
WORKDIR /app
ENV NODE_ENV=production
RUN adduser -D app && chown -R app:app /app
USER app
COPY --from=deps /app/node_modules /app/node_modules
COPY . .
EXPOSE 8080
CMD ["node","server.mjs"]
