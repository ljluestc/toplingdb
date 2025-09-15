//! Cache benchmarks

use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId};
use toplingdb::cache::{Cache, LRUCache, NoCache};

fn bench_lru_cache_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_cache_insert");

    for capacity in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            capacity,
            |b, &capacity| {
                b.iter_batched(
                    || {
                        LRUCache::new(capacity)
                    },
                    |cache| {
                        for i in 0..capacity {
                            let key = format!("key_{}", i);
                            let value = vec![b'x'; 100];
                            cache.insert(key.as_bytes(), value, 100);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_lru_cache_get_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_cache_get_hit");

    for capacity in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            capacity,
            |b, &capacity| {
                // Setup: populate cache
                let cache = LRUCache::new(capacity);
                for i in 0..capacity {
                    let key = format!("key_{}", i);
                    let value = vec![b'x'; 100];
                    cache.insert(key.as_bytes(), value, 100);
                }

                b.iter(|| {
                    let key_idx = fastrand::usize(0..capacity);
                    let key = format!("key_{}", key_idx);
                    let _result = cache.lookup(key.as_bytes());
                });
            },
        );
    }

    group.finish();
}

fn bench_lru_cache_get_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_cache_get_miss");

    for capacity in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            capacity,
            |b, &capacity| {
                // Setup: populate cache
                let cache = LRUCache::new(capacity);
                for i in 0..capacity {
                    let key = format!("key_{}", i);
                    let value = vec![b'x'; 100];
                    cache.insert(key.as_bytes(), value, 100);
                }

                b.iter(|| {
                    let key = format!("missing_key_{}", fastrand::u64(..));
                    let _result = cache.lookup(key.as_bytes());
                });
            },
        );
    }

    group.finish();
}

fn bench_lru_cache_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_cache_eviction");

    group.bench_function("sequential_eviction", |b| {
        let capacity = 1000;
        b.iter_batched(
            || {
                LRUCache::new(capacity)
            },
            |cache| {
                // Insert more items than capacity to trigger eviction
                for i in 0..(capacity * 2) {
                    let key = format!("key_{}", i);
                    let value = vec![b'x'; 100];
                    cache.insert(key.as_bytes(), value, 100);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_lru_cache_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_cache_mixed_workload");

    group.bench_function("read_heavy", |b| {
        let capacity = 1000;
        let cache = LRUCache::new(capacity);

        // Populate with initial data
        for i in 0..capacity {
            let key = format!("key_{}", i);
            let value = vec![b'x'; 100];
            cache.insert(key.as_bytes(), value, 100);
        }

        b.iter(|| {
            for _ in 0..100 {
                // 90% reads, 10% writes
                if fastrand::f32() < 0.9 {
                    let key_idx = fastrand::usize(0..capacity);
                    let key = format!("key_{}", key_idx);
                    let _result = cache.lookup(key.as_bytes());
                } else {
                    let key_idx = fastrand::usize(0..capacity);
                    let key = format!("key_{}", key_idx);
                    let value = vec![b'y'; 100];
                    cache.insert(key.as_bytes(), value, 100);
                }
            }
        });
    });

    group.finish();
}

fn bench_no_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("no_cache");

    group.bench_function("no_cache_operations", |b| {
        b.iter(|| {
            let cache = NoCache;

            // Simulate cache operations
            for i in 0..1000 {
                let key = format!("key_{}", i);
                let value = vec![b'x'; 100];
                cache.insert(key.as_bytes(), value, 100);
                let _result = cache.lookup(key.as_bytes());
            }
        });
    });

    group.finish();
}

fn bench_cache_size_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_size_tracking");

    group.bench_function("size_operations", |b| {
        let capacity = 1000;
        let cache = LRUCache::new(capacity);

        b.iter(|| {
            // Insert some items
            for i in 0..100 {
                let key = format!("key_{}", i);
                let value = vec![b'x'; 50];
                cache.insert(key.as_bytes(), value, 50);
            }

            // Check size and other properties
            let _usage = cache.get_usage();
            let _capacity = cache.get_capacity();
            let _entry_count = cache.get_entry_count();

            // Clear some items
            cache.clear();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lru_cache_insert,
    bench_lru_cache_get_hit,
    bench_lru_cache_get_miss,
    bench_lru_cache_eviction,
    bench_lru_cache_mixed_workload,
    bench_no_cache,
    bench_cache_size_tracking
);

criterion_main!(benches);