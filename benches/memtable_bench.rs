//! MemTable benchmarks

use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId};
use toplingdb::memtable::MemTable;
use toplingdb::options::ColumnFamilyOptions;
use toplingdb::ValueType;

fn create_memtable() -> MemTable {
    let options = ColumnFamilyOptions::default();
    MemTable::new(options)
}

fn bench_memtable_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_put");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let memtable = create_memtable();
                        let key = format!("key_{}", size);
                        let value = vec![b'x'; size];
                        (memtable, key, value)
                    },
                    |(memtable, key, value)| {
                        memtable.insert(key.as_bytes(), &value, ValueType::Value, 1).unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_memtable_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_get");

    for num_keys in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            num_keys,
            |b, &num_keys| {
                // Setup: populate memtable
                let memtable = create_memtable();

                for i in 0..num_keys {
                    let key = format!("key_{:06}", i);
                    let value = format!("value_{:06}", i);
                    memtable.insert(key.as_bytes(), value.as_bytes(), ValueType::Value, i as u64).unwrap();
                }

                b.iter(|| {
                    let key_idx = fastrand::usize(0..num_keys);
                    let key = format!("key_{:06}", key_idx);
                    let _result = memtable.get(key.as_bytes());
                });
            },
        );
    }

    group.finish();
}

fn bench_memtable_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_delete");

    for num_keys in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            num_keys,
            |b, &num_keys| {
                b.iter_batched(
                    || {
                        let memtable = create_memtable();

                        // Populate memtable
                        for i in 0..num_keys {
                            let key = format!("key_{:06}", i);
                            let value = format!("value_{:06}", i);
                            memtable.insert(key.as_bytes(), value.as_bytes(), ValueType::Value, i as u64).unwrap();
                        }

                        memtable
                    },
                    |memtable| {
                        for i in 0..num_keys {
                            let key = format!("key_{:06}", i);
                            memtable.insert(key.as_bytes(), &[], ValueType::Delete, (i + num_keys) as u64).unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_memtable_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_iteration");

    for num_keys in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            num_keys,
            |b, &num_keys| {
                // Setup: populate memtable
                let memtable = create_memtable();

                for i in 0..num_keys {
                    let key = format!("key_{:06}", i);
                    let value = format!("value_{:06}", i);
                    memtable.insert(key.as_bytes(), value.as_bytes(), ValueType::Value, i as u64).unwrap();
                }

                b.iter(|| {
                    let mut iter = memtable.iter();
                    iter.seek_to_first();
                    let mut count = 0;
                    while iter.valid() {
                        count += 1;
                        iter.next();
                    }
                    count
                });
            },
        );
    }

    group.finish();
}

fn bench_memtable_size_estimation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_size_estimation");

    group.bench_function("approximate_size", |b| {
        let memtable = create_memtable();

        // Populate with test data
        for i in 0..1000 {
            let key = format!("key_{:06}", i);
            let value = format!("value_{:06}", i);
            memtable.insert(key.as_bytes(), value.as_bytes(), ValueType::Value, i as u64).unwrap();
        }

        b.iter(|| {
            let start_key = b"key_000000";
            let end_key = b"key_999999";
            memtable.approximate_size(start_key, end_key)
        });
    });

    group.finish();
}

fn bench_memtable_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memtable_memory_usage");

    group.bench_function("memory_usage", |b| {
        let memtable = create_memtable();

        // Populate with test data
        for i in 0..1000 {
            let key = format!("key_{:06}", i);
            let value = format!("value_{:06}", i);
            memtable.insert(key.as_bytes(), value.as_bytes(), ValueType::Value, i as u64).unwrap();
        }

        b.iter(|| {
            memtable.memory_usage()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memtable_put,
    bench_memtable_get,
    bench_memtable_delete,
    bench_memtable_iteration,
    bench_memtable_size_estimation,
    bench_memtable_memory_usage
);

criterion_main!(benches);