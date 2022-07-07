#![feature(test)]
#![feature(extend_one)]

extern crate test;

use buter::Buter;
use criterion::{criterion_group, criterion_main, Criterion};
use test::{black_box, Bencher};

const SECRET_N: usize = 1_000_000;

fn callback_work<F>(mut f: F)
where
    F: FnMut(usize),
{
    for i in 0..SECRET_N {
        f(i);
    }
}

#[inline(always)]
fn silent_box<T>(t: T) {
    black_box(t);
}

fn vec_push(c: &mut Criterion) {
    fn iter() -> impl Iterator<Item = usize> {
        let mut vec = Vec::new();
        callback_work(|i| vec.push(i));
        vec.into_iter()
    }

    c.bench_function("vec_push", |b| b.iter(|| iter().for_each(silent_box)));
}

fn vec_push_with_capacity(c: &mut Criterion) {
    fn iter() -> impl Iterator<Item = usize> {
        let mut vec = Vec::with_capacity(SECRET_N);
        callback_work(|i| vec.push(i));
        vec.into_iter()
    }

    c.bench_function("vec_push_with_capacity", |b| {
        b.iter(|| iter().for_each(silent_box))
    });
}

fn buter(c: &mut Criterion) {
    struct Helper(pub Buter<usize>);
    impl Helper {
        fn work(&self) -> impl Iterator<Item = usize> + '_ {
            let mut writer = self.0.writer();
            callback_work(|i| writer.extend_one(i));
            writer.into_iter()
        }
    }

    let mut helper = Helper(Buter::new());
    c.bench_function("buter", |b| b.iter(|| helper.work().for_each(silent_box)));
}

criterion_group!(benches, vec_push, vec_push_with_capacity, buter);
criterion_main!(benches);
