//! Database benchmarks

use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId};
use tempfile::TempDir;
use toplingdb::{DB, Options, ReadOptions, WriteOptions, WriteBatch};

fn create_db() -> (DB, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("benchmark_db");

    let mut options = Options::default();
    options.create_if_missing(true);
    options.write_buffer_size(64 * 1024 * 1024); // 64MB

    let db = DB::open(&options, &path).unwrap();
    (db, temp_dir)
}

fn bench_put_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_single");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let (db, _temp_dir) = create_db();
                        let write_opts = WriteOptions::default();
                        let key = format!("key_{}", size);
                        let value = vec![b'x'; size];
                        (db, _temp_dir, write_opts, key, value)
                    },
                    |(db, _temp_dir, write_opts, key, value)| {
                        db.put(&write_opts, key.as_bytes(), &value).unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_put_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_batch");

    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter_batched(
                    || {
                        let (db, _temp_dir) = create_db();
                        let write_opts = WriteOptions::default();
                        let mut batch = WriteBatch::new();

                        for i in 0..batch_size {
                            let key = format!("key_{}", i);
                            let value = format!("value_{}", i);
                            batch.put(key.as_bytes(), value.as_bytes());
                        }

                        (db, _temp_dir, write_opts, batch)
                    },
                    |(db, _temp_dir, write_opts, batch)| {
                        db.write(&write_opts, &batch).unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_get_existing(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_existing");

    for num_keys in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            num_keys,
            |b, &num_keys| {
                // Setup: populate database
                let (db, _temp_dir) = create_db();
                let write_opts = WriteOptions::default();
                let read_opts = ReadOptions::default();

                for i in 0..num_keys {
                    let key = format!("key_{:06}", i);
                    let value = format!("value_{:06}", i);
                    db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
                }

                b.iter(|| {
                    let key_idx = fastrand::usize(0..num_keys);
                    let key = format!("key_{:06}", key_idx);
                    let _result = db.get(&read_opts, key.as_bytes()).unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_get_missing(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_missing");

    for num_keys in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            num_keys,
            |b, &num_keys| {
                // Setup: populate database
                let (db, _temp_dir) = create_db();
                let write_opts = WriteOptions::default();
                let read_opts = ReadOptions::default();

                for i in 0..num_keys {
                    let key = format!("key_{:06}", i);
                    let value = format!("value_{:06}", i);
                    db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
                }

                b.iter(|| {
                    let key = format!("missing_key_{}", fastrand::u64(..));
                    let _result = db.get(&read_opts, key.as_bytes()).unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    group.bench_function("read_heavy", |b| {
        let (db, _temp_dir) = create_db();
        let write_opts = WriteOptions::default();
        let read_opts = ReadOptions::default();

        // Populate with initial data
        for i in 0..1000 {
            let key = format!("key_{:06}", i);
            let value = format!("value_{:06}", i);
            db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
        }

        b.iter(|| {
            for _ in 0..10 {
                // 90% reads, 10% writes
                if fastrand::f32() < 0.9 {
                    let key_idx = fastrand::usize(0..1000);
                    let key = format!("key_{:06}", key_idx);
                    let _result = db.get(&read_opts, key.as_bytes()).unwrap();
                } else {
                    let key_idx = fastrand::usize(0..1000);
                    let key = format!("key_{:06}", key_idx);
                    let value = format!("updated_value_{:06}", key_idx);
                    db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
                }
            }
        });
    });

    group.finish();
}

fn bench_sequential_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_write");

    group.bench_function("sequential_keys", |b| {
        b.iter_batched(
            || {
                let (db, _temp_dir) = create_db();
                let write_opts = WriteOptions::default();
                (db, _temp_dir, write_opts)
            },
            |(db, _temp_dir, write_opts)| {
                for i in 0..1000 {
                    let key = format!("key_{:08}", i);
                    let value = vec![b'x'; 100];
                    db.put(&write_opts, key.as_bytes(), &value).unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_random_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_write");

    group.bench_function("random_keys", |b| {
        b.iter_batched(
            || {
                let (db, _temp_dir) = create_db();
                let write_opts = WriteOptions::default();
                (db, _temp_dir, write_opts)
            },
            |(db, _temp_dir, write_opts)| {
                for _ in 0..1000 {
                    let key = format!("key_{:08}", fastrand::u64(..));
                    let value = vec![b'x'; 100];
                    db.put(&write_opts, key.as_bytes(), &value).unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_put_single,
    bench_put_batch,
    bench_get_existing,
    bench_get_missing,
    bench_mixed_workload,
    bench_sequential_write,
    bench_random_write
);

criterion_main!(benches);