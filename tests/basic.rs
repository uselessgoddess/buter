#![feature(extend_one)]

use buter::Buter;

#[test]
fn basic() {
    let mut buf = Buter::new();
    buf.extend_one(1);
    assert_eq!(buf.iter().next(), Some(1));
}

fn callback_work<F>(mut f: F)
where
    F: FnMut(usize),
{
    for i in 0..10 {
        f(i);
    }
}

struct BasicHelper {
    buf: Buter<usize>,
}

impl BasicHelper {
    pub fn new() -> Self {
        BasicHelper { buf: Buter::new() }
    }

    pub fn work(&mut self) -> impl Iterator<Item = usize> + '_ {
        callback_work(|i| self.buf.extend_one(i));
        self.buf.iter()
    }
}

#[test]
fn in_struct() {
    let mut helper = BasicHelper::new();

    assert!(helper.work().eq(0..10));
}
