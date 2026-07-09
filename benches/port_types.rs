//! Criterion benchmarks for `phenocompose-port-types` value types.

use criterion::{criterion_group, criterion_main, Criterion};
use phenocompose_port_types::{Manifest, PortError, Secret, SecretRef};

fn manifest_new(c: &mut Criterion) {
    c.bench_function("manifest_new", |b| {
        b.iter(|| {
            let _ = Manifest {
                name: "bench-sandbox".to_string(),
                artifact_name: None,
                tags: vec![],
            };
        });
    });
}

fn manifest_clone(c: &mut Criterion) {
    let m = Manifest {
        name: "src".to_string(),
        artifact_name: Some("src:v1".to_string()),
        tags: vec![("env".to_string(), "prod".to_string())],
    };
    c.bench_function("manifest_clone", |b| {
        b.iter(|| {
            let _ = m.clone();
        });
    });
}

fn secret_ref_locator(c: &mut Criterion) {
    let s = SecretRef::namespaced("phenotype", "db-password");
    c.bench_function("secret_ref_locator", |b| {
        b.iter(|| {
            let _ = s.locator();
        });
    });
}

fn port_error_display(c: &mut Criterion) {
    let e = PortError::Validation("bad input".to_string());
    c.bench_function("port_error_display", |b| {
        b.iter(|| {
            let _ = format!("{}", e);
        });
    });
}

fn secret_new(c: &mut Criterion) {
    c.bench_function("secret_new", |b| {
        b.iter(|| {
            let _ = Secret::new(SecretRef::new("k"), "value");
        });
    });
}

criterion_group!(benches, manifest_new, manifest_clone, secret_ref_locator, port_error_display, secret_new);
criterion_main!(benches);
