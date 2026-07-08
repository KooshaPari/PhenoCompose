# ADR 0003: Port Trait Design

## Status

Accepted. 2026-07-08.

## Context

PhenoCompose implements the hex-architecture port pattern. Four port traits (Composer, Publisher, Runtime, SecretStore) define the boundary between domain logic and adapter implementations.

## Decision

Each port trait follows these rules:

1. **Object-safe**: no associated types, no generic methods, only `&self` receivers
2. **Send + Sync**: required for `Box<dyn Trait>` storage and cross-thread dispatch
3. **Transport-agnostic**: no file paths, URIs, or environment variables in method signatures
4. **Domain types only**: methods take/return port-types value types (Manifest, ComposedArtifact, etc.)

## Consequences

- Adapters (Apple Container, WSL2, file, in-memory, etc.) can be swapped via `Box<dyn PortTrait>`
- DI container (port-di) wires adapters into the composition pipeline
- Error handling: each port has a single error type (e.g., `PortError`) that adapters can map to
