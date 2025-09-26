# ---------- Build ----------
FROM node:20.14-alpine AS build
WORKDIR /app

# Pass API URL at build time (optional fallback)
ARG VITE_API_URL=http://backend:8080
ENV VITE_API_URL=${VITE_API_URL}

# Install deps with cache
COPY package*.json ./
RUN npm ci --no-audit --no-fund

# Copy sources & build
COPY . .
# (Vite reads VITE_* envs at build time)
RUN npm run build

# ---------- Runtime (Nginx) ----------
FROM nginx:1.27-alpine
# SPA fallback & gzip/etc. — make sure your nginx.conf supports history mode
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Static assets
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80
# No daemon mode; log to stdout/stderr
CMD ["nginx", "-g", "daemon off;"]
