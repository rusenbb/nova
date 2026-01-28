//! Performance benchmarks for Nova.
//!
//! Run with: cargo bench
//!
//! Target performance:
//! - App discovery: < 500ms
//! - Search latency: < 50ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nova::platform;
use nova::services::calculator;

/// Benchmark app discovery across the platform.
fn bench_app_discovery(c: &mut Criterion) {
    let platform = platform::current();

    c.bench_function("platform_discover_apps", |b| {
        b.iter(|| {
            let apps = platform.discover_apps();
            black_box(apps)
        })
    });
}

/// Benchmark directory path resolution.
fn bench_directory_paths(c: &mut Criterion) {
    let platform = platform::current();

    let mut group = c.benchmark_group("directory_paths");

    group.bench_function("config_dir", |b| {
        b.iter(|| black_box(platform.config_dir()))
    });

    group.bench_function("data_dir", |b| b.iter(|| black_box(platform.data_dir())));

    group.bench_function("runtime_dir", |b| {
        b.iter(|| black_box(platform.runtime_dir()))
    });

    group.finish();
}

/// Benchmark calculator evaluation.
fn bench_calculator(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator");

    let expressions = [
        ("simple_add", "2 + 2"),
        ("complex_expr", "(10 + 5) * 2 / 3"),
        ("power", "2^10"),
        ("sqrt", "sqrt(144)"),
        ("scientific", "sin(3.14159) + cos(0)"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(name), &expr, |b, expr| {
            b.iter(|| black_box(calculator::evaluate(black_box(expr))))
        });
    }

    group.finish();
}

/// Benchmark fuzzy search operations (simulated with string matching).
fn bench_fuzzy_search(c: &mut Criterion) {
    // Create test data similar to what we'd have in app discovery
    let test_entries: Vec<String> = (0..200)
        .map(|i| format!("Application {} - Test Entry for Benchmarking", i))
        .collect();

    let queries = ["app", "test", "application 50", "bench"];

    let mut group = c.benchmark_group("fuzzy_search");

    for query in queries {
        group.bench_with_input(BenchmarkId::from_parameter(query), &query, |b, query| {
            b.iter(|| {
                let query_lower = query.to_lowercase();
                let results: Vec<&String> = test_entries
                    .iter()
                    .filter(|entry| entry.to_lowercase().contains(&query_lower))
                    .take(8)
                    .collect();
                black_box(results)
            })
        });
    }

    group.finish();
}

/// Benchmark clipboard operations (only when not in CI).
fn bench_clipboard(c: &mut Criterion) {
    // Skip clipboard benchmarks if in CI or no display available
    if std::env::var("CI").is_ok() {
        return;
    }

    let platform = platform::current();

    let mut group = c.benchmark_group("clipboard");

    // Benchmark clipboard read (may be empty)
    group.bench_function("read", |b| b.iter(|| black_box(platform.clipboard_read())));

    // Benchmark clipboard write
    let test_content = "Nova benchmark test content 🎉";
    group.bench_function("write", |b| {
        b.iter(|| black_box(platform.clipboard_write(black_box(test_content))))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_app_discovery,
    bench_directory_paths,
    bench_calculator,
    bench_fuzzy_search,
    bench_clipboard,
);

criterion_main!(benches);
