use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kylos_qpadl::bench::{bench_keygen, bench_sign, bench_verify, ITERATIONS, WARMUP};

fn criterion_benchmark(c: &mut Criterion) {
    kylos_qpadl::init();

    let mut group = c.benchmark_group("MAYO");
    group.sample_size(100);
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(5));

    for level in kylos_qpadl::supported_levels() {
        let sig = kylos_qpadl::create_sig(level).unwrap();
        let msg = b"Kylos Arc QPADL benchmark message";

        group.bench_function(format!("MAYO-{level}/keygen"), |b| {
            b.iter(|| {
                let _ = black_box(sig.keypair());
            });
        });

        let (pk, sk) = sig.keypair().unwrap();
        let signature = sig.sign(msg, &sk).unwrap();

        group.bench_function(format!("MAYO-{level}/sign"), |b| {
            b.iter(|| {
                let _ = black_box(sig.sign(black_box(msg), &sk));
            });
        });

        group.bench_function(format!("MAYO-{level}/verify"), |b| {
            b.iter(|| {
                let _ = black_box(sig.verify(black_box(msg), black_box(&signature), black_box(&pk)));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
