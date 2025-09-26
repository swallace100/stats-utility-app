# Architecture

The Stats Utility App is composed of four main services:

1. **Frontend (React)**

   - Vite + Tailwind + shadcn/ui
   - User uploads CSV, selects analysis, views results

2. **Backend (Node.js)**

   - Orchestrates jobs, manages datasets
   - Communicates with Rust + Python services

3. **Rust Microservice**

   - Statistical kernels (t-test, ANOVA, regression, etc.)
   - Deterministic JSON outputs

4. **Python Microservice**
   - FastAPI + matplotlib
   - Turns chart specs into PNG/SVG

All services run via Docker Compose.
