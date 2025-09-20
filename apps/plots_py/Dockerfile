# apps/plots_py/Dockerfile

# --- base image with OS deps ---
FROM python:3.12-slim AS base
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfreetype6 libjpeg62-turbo libpng16-16 wget curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# --- production runtime ---
FROM base AS runtime
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
ENV MPLBACKEND=Agg  
COPY . .
EXPOSE 7000
HEALTHCHECK --interval=5s --timeout=3s --retries=12 CMD wget -qO- http://127.0.0.1:7000/health || exit 1
CMD ["uvicorn","main:app","--host","0.0.0.0","--port","7000"]

# --- test stage (what compose.test.yml targets) ---
FROM base AS test
COPY requirements.txt requirements-dev.txt ./
RUN pip install --no-cache-dir -r requirements-dev.txt
COPY . .
# only run the test runner here
CMD ["pytest","-q"]
