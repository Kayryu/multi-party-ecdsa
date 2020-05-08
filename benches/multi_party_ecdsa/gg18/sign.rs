use criterion::criterion_main;

mod bench {
    use criterion::{criterion_group, Criterion};
    use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2018::mock::sign as mpc_sign;

    pub fn bench_full_keygen_party_two_five(c: &mut Criterion) {
        c.bench_function("keygen t=2 n=5", move |b| {
            b.iter(|| {
                mpc_sign(2, 5, 4, vec![0, 2, 3, 4])
            })
        });
    }

    pub fn bench_full_keygen_party_four_eight(c: &mut Criterion) {
        c.bench_function("keygen t=4 n=8", move |b| {
            b.iter(|| {
                mpc_sign(4, 8, 6, vec![0, 1, 2, 4, 6, 7])
            })
        });
    }

    pub fn bench_full_keygen_party_sixteen_twenty(c: &mut Criterion) {
        c.bench_function("keygen t=16 n=20", move |b| {
            b.iter(|| {
                mpc_sign(16, 20, 17, vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 18, 19]);
            })
        });
    }

    criterion_group! {
    name = sign;
    config = Criterion::default().sample_size(10);
    targets =
    self::bench_full_keygen_party_two_five,
    self::bench_full_keygen_party_four_eight,
    self::bench_full_keygen_party_sixteen_twenty}
}

criterion_main!(bench::sign);
