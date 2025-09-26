# Contributing Guide

Thank you for your interest in contributing 🎉

## Getting Started

- Clone the repo
- Install Docker + Node + Rust + Python
- Copy `.env.example` to `.env`

## Development

### Run the full stack

```bash
docker compose up -d
```

### Run frontend only

```bash
npm run dev --workspace=frontend
```

### Run backend only

```bash
npm run dev --workspace=backend
```

### Run Rust microservice

```bash
cargo run --manifest-path=stats_rs/Cargo.toml
```

### Run Python plotting service

```bash
uvicorn plots_py.main:app --reload --port 7000
```

### Run tests

- Rust tests:
  ```bash
  cargo test
  ```
- Node tests:
  ```bash
  npm test --workspace=backend
  ```
- Python tests (if added):
  ```bash
  pytest
  ```

## Coding Standards

- TypeScript: use `eslint` + `prettier`
- Rust: run `cargo fmt`
- Python: use `black` and `mypy`

## Submitting a PR

1. Fork and branch from `main`
2. Write clear commit messages
3. Run tests before pushing
4. Open a PR using the [PR template](./.github/pull_request_template.md)

## Reporting Issues

Use GitHub Issues and include:

- Expected vs actual behavior
- Steps to reproduce
- Logs or error output
