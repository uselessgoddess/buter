#![feature(extend_one)]

use buter::Buter;

#[test]
fn basic() {
    let buf = Buter::new();
    let mut writer = buf.writer();

    writer.extend_one(1);

    assert_eq!(writer.into_iter().next(), Some(1));
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

    pub fn work(&self) -> impl Iterator<Item = usize> + '_ {
        let mut writer = self.buf.writer();
        callback_work(|i| writer.extend_one(i));
        writer.into_iter()
    }
}

#[test]
fn in_struct() {
    let helper = BasicHelper::new();

    assert!(helper.work().eq(0..10));
}
