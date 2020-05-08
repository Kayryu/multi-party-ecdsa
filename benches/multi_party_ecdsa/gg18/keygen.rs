use criterion::criterion_main;

mod bench {
    use criterion::{criterion_group, Criterion};
    use curv::cryptographic_primitives::secret_sharing::feldman_vss::VerifiableSS;
    use curv::elliptic::curves::traits::*;
    use curv::{FE, GE};
    use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2018::mock::*;

    pub fn bench_full_keygen_party_one_two(c: &mut Criterion) {
        c.bench_function("keygen t=1 n=2", move |b| {
            b.iter(|| {
                keygen_t_n_parties(1, 2);
            })
        });
    }
    pub fn bench_full_keygen_party_two_three(c: &mut Criterion) {
        c.bench_function("keygen t=2 n=3", move |b| {
            b.iter(|| {
                keygen_t_n_parties(2, 3);
            })
        });
    }

    pub fn bench_full_keygen_party_two_five(c: &mut Criterion) {
        c.bench_function("keygen t=2 n=5", move |b| {
            b.iter(|| {
                keygen_t_n_parties(2, 5);
            })
        });
    }
    pub fn bench_full_keygen_party_four_eight(c: &mut Criterion) {
        c.bench_function("keygen t=4 n=8", move |b| {
            b.iter(|| {
                keygen_t_n_parties(4, 8);
            })
        });
    }
    pub fn bench_full_keygen_party_sixteen_twenty(c: &mut Criterion) {
        c.bench_function("keygen t=16 n=20", move |b| {
            b.iter(|| {
                keygen_t_n_parties(4, 8);
            })
        });
    }

    criterion_group! {
    name = keygen;
    config = Criterion::default().sample_size(10);
    targets =
    self::bench_full_keygen_party_one_two,
    self::bench_full_keygen_party_two_three,
    self::bench_full_keygen_party_two_five,
    self::bench_full_keygen_party_four_eight,
    self::bench_full_keygen_party_sixteen_twenty}
}

criterion_main!(bench::keygen);
