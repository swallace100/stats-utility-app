# apps/plots_py/Dockerfile

# --- base image with OS deps ---
FROM python:3.13-slim AS base
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libfreetype6 libjpeg62-turbo libpng16-16 wget curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# --- production runtime ---
FROM base AS runtime
# Install runtime deps first for better layer caching
COPY requirements.txt /app/plots_py/requirements.txt
RUN pip install --no-cache-dir -r /app/plots_py/requirements.txt

# Matplotlib backend for headless
ENV MPLBACKEND=Agg

# Copy the service code (build context is apps/plots_py)
COPY . /app/plots_py

# Ensure package exists (safe if already present)
RUN python -c "import pathlib; pathlib.Path('/app/plots_py/__init__.py').touch()"

# Make the package importable
ENV PYTHONPATH=/app

# Create output/cache dir so your volume mount has a target
RUN mkdir -p /app/out

EXPOSE 7000

# Run as a package module so relative imports (from .models) work
CMD ["uvicorn","plots_py.main:app","--host","0.0.0.0","--port","7000"]

# --- test stage (optional; only used if your compose targets it) ---
FROM base AS test
COPY requirements.txt requirements-dev.txt /app/plots_py/
RUN pip install --no-cache-dir -r /app/plots_py/requirements-dev.txt
COPY . /app/plots_py
RUN python -c "import pathlib; pathlib.Path('/app/plots_py/__init__.py').touch()"
ENV PYTHONPATH=/app
WORKDIR /app
CMD ["pytest","-q","plots_py/tests"]
