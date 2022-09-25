use bcrypt::DEFAULT_COST;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
pub async fn hash_password(password: String) -> String
{
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result = bcrypt::hash(password, DEFAULT_COST);
        let _ = send.send(result);
    });
    recv.await.unwrap().unwrap()
}

fn criterion_benchmark(c: &mut Criterion)
{
    c.bench_function("bcrypt 12 cost 15char password hashing", |b| {
        b.iter(|| {
            hash_password(black_box(
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(15)
                    .map(char::from)
                    .collect(),
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
