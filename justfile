# justfile for PhenoCompose
# Replaces Taskfile.yml. Use `just` (or `just <recipe>`) to run recipes.
# `just` is the casey/just command runner: https://just.systems

set shell := ["bash", "-uc"]
set dotenv-load

# ---- Detected features (eval once, exported as env vars) ----

export HAS_GO := `test -f go.mod && echo 1 || echo 0`
export GO_PACKAGES := `if [ -d cmd ] || [ -d internal ]; then echo "./cmd/... ./internal/..."; fi`
export HAS_RUST := `cargo metadata --manifest-path pheno-compose-driver/Cargo.toml --no-deps --format-version 1 >/dev/null 2>&1 && echo 1 || echo 0`
export HAS_RUST_FFI := `cargo metadata --manifest-path bindings/rust-ffi/Cargo.toml --no-deps --format-version 1 >/dev/null 2>&1 && echo 1 || echo 0`
export HAS_ROOT_PACKAGE := `test -f package.json && echo 1 || echo 0`
export HAS_DOCS_BUILD := `node -e 'const fs=require("fs"); const p=JSON.parse(fs.readFileSync("package.json", "utf8")); process.exit(p.scripts && p.scripts["docs:build"] ? 0 : 1)' >/dev/null 2>&1 && echo 1 || echo 0`
export HAS_PLAYWRIGHT := `test -f tests/playwright/package.json && echo 1 || echo 0`
export HAS_PLAYWRIGHT_BUILD := `node -e 'const fs=require("fs"); const p=JSON.parse(fs.readFileSync("tests/playwright/package.json", "utf8")); process.exit(p.scripts && p.scripts.build ? 0 : 1)' >/dev/null 2>&1 && echo 1 || echo 0`
export HAS_PLAYWRIGHT_TEST := `node -e 'const fs=require("fs"); const p=JSON.parse(fs.readFileSync("tests/playwright/package.json", "utf8")); process.exit(p.scripts && p.scripts.test ? 0 : 1)' >/dev/null 2>&1 && echo 1 || echo 0`
export JS_RUNNER := `command -v bun >/dev/null 2>&1 && echo bun || echo npm`

# ---- Default recipe: list available recipes (like `task --list`) ----

default: list

# Show all available recipes
list:
    @just --list

# ---- Build: build repo components detected from manifests ----

build:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_GO}" = "1" ]; then
      go build ${GO_PACKAGES}
    fi

    if [ "${HAS_RUST}" = "1" ]; then
      cargo build --manifest-path pheno-compose-driver/Cargo.toml
    fi

    if [ "${HAS_RUST_FFI}" = "1" ]; then
      cargo build --manifest-path bindings/rust-ffi/Cargo.toml
    fi

    if [ "${HAS_DOCS_BUILD}" = "1" ]; then
      ${JS_RUNNER} run docs:build
    fi

    if [ "${HAS_PLAYWRIGHT}" = "1" ] && [ "${HAS_PLAYWRIGHT_BUILD}" = "1" ]; then
      (cd tests/playwright && ${JS_RUNNER} run build)
    fi

# ---- Test: run repo tests for detected components ----

test:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_GO}" = "1" ]; then
      go test ${GO_PACKAGES}
    fi

    if [ "${HAS_RUST}" = "1" ]; then
      cargo test --manifest-path pheno-compose-driver/Cargo.toml
    fi

    if [ "${HAS_RUST_FFI}" = "1" ]; then
      cargo test --manifest-path bindings/rust-ffi/Cargo.toml
    fi

    if [ "${HAS_PLAYWRIGHT}" = "1" ] && [ "${HAS_PLAYWRIGHT_TEST}" = "1" ]; then
      (cd tests/playwright && ${JS_RUNNER} run test)
    fi

# ---- Lint: lint repo components detected from manifests ----

lint:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_GO}" = "1" ]; then
      if command -v golangci-lint >/dev/null 2>&1; then
        golangci-lint run ${GO_PACKAGES}
      else
        go vet ${GO_PACKAGES}
      fi
    fi

    if [ "${HAS_RUST}" = "1" ]; then
      cargo fmt --manifest-path pheno-compose-driver/Cargo.toml --all --check
      cargo clippy --manifest-path pheno-compose-driver/Cargo.toml --all-targets --all-features -- -D warnings
    fi

    if [ "${HAS_RUST_FFI}" = "1" ]; then
      cargo fmt --manifest-path bindings/rust-ffi/Cargo.toml --all --check
      cargo clippy --manifest-path bindings/rust-ffi/Cargo.toml --all-targets --all-features -- -D warnings
    fi

    if [ "${HAS_DOCS_BUILD}" = "1" ]; then
      ${JS_RUNNER} run docs:build
    fi

# ---- Clean: remove generated artifacts for detected components ----

clean:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_GO}" = "1" ]; then
      go clean ${GO_PACKAGES}
    fi

    if [ "${HAS_RUST}" = "1" ]; then
      rm -rf pheno-compose-driver/target
    fi

    if [ "${HAS_RUST_FFI}" = "1" ]; then
      rm -rf bindings/rust-ffi/target
    fi

    if [ "${HAS_ROOT_PACKAGE}" = "1" ]; then
      rm -rf docs/.vitepress/dist
    fi

    if [ "${HAS_PLAYWRIGHT}" = "1" ]; then
      rm -rf tests/playwright/dist tests/playwright/node_modules/.cache
    fi

# Measure code coverage (SSOT: see grade.sh for the canonical command)
coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -f "Cargo.toml" ]]; then
        cargo llvm-cov --workspace --fail-under-lines 85
    elif [[ -f "package.json" ]]; then
        npx jest --coverage --coverageThreshold='{"global":{"branches":85,"functions":85,"lines":85,"statements":85}}'
    elif [[ -f "pyproject.toml" || -f "setup.py" ]]; then
        pytest --cov=src --cov-report=term-missing --cov-fail-under=85
    elif [[ -f "go.mod" ]]; then
        go test -coverprofile=coverage.out -covermode=atomic ./... && go tool cover -func=coverage.out | grep total | awk '{print $3}' | sed 's/%//' | awk '{exit($1 < 85 ? 1 : 0)}'
    else
        echo "No recognized stack (Cargo.toml / package.json / pyproject.toml / go.mod) found." >&2
        exit 1
    fi
