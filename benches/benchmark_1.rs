use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use tempfile::TempDir;
use rand::prelude::*;

pub fn set_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.significance_level(0.1).sample_size(500);
    group
        .bench_function("kvs",
                        |b| {
                            b.iter_batched(
                                || {
                                    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
                                    (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
                                },
                                |(mut store, _tem_dir)| {
                                    for i in 1..(1 << 12) {
                                        store.set(format!("key{}", i), "value".to_string()).unwrap();
                                    }
                                },
                                BatchSize::SmallInput,
                            )
                        },
        )
        .bench_function("sled",
                        |b| {
                            b.iter_batched(
                                || {
                                    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
                                    (SledKvsEngine::open(temp_dir.path()).unwrap(), temp_dir)
                                },
                                |(mut db, _tem_dir)| {
                                    for i in 1..(1 << 12) {
                                        db.set(format!("key{}", i), "value".to_string()).unwrap();
                                    }
                                },
                                BatchSize::SmallInput,
                            )
                        },
        );
    group.finish();
}

pub fn get_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_bench");
    group.significance_level(0.1).sample_size(500);
    let input = vec![8];
    for i in input {
        group
            .bench_with_input(BenchmarkId::new("kvs", i),
                              &i,
                              |b, j| {
                                  let temp_dir = TempDir::new().expect("unable to create temporary working directory");
                                  let mut store = KvStore::open(temp_dir.path()).unwrap();
                                  for key_i in 1..(1 << j) {
                                      store.set(format!("key{}", key_i), "value".to_string()).unwrap();
                                  }
                                  let mut rng = SmallRng::from_seed([0;16]);
                                  b.iter(||store.get(format!("key{}", rng.gen_range(1, 1 << j))).unwrap());
                              });
        group
            .bench_with_input(BenchmarkId::new("sled", i),
                              &i,
                              |b, j| {
                                  let temp_dir = TempDir::new().expect("unable to create temporary working directory");
                                  let mut store = SledKvsEngine::open(temp_dir.path()).unwrap();
                                  for key_i in 1..(1 << j) {
                                      store.set(format!("key{}", key_i), "value".to_string()).unwrap();
                                  }
                                  let mut rng = SmallRng::from_seed([0;16]);
                                  b.iter(||store.get(format!("key{}", rng.gen_range(1, 1 << j))).unwrap());
                              });
    }
    group.finish();
}

criterion_group!(benches, get_benchmark, set_benchmark);
criterion_main!(benches);
