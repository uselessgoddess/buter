use buter::Buter;
use std::marker::PhantomPinned;

struct Pinned {
    _pin: PhantomPinned,
}

fn main() {
    let buf = Buter::new();
    let mut writer = buf.writer();
    writer.extend(Some(Pinned {
        _pin: PhantomPinned,
    }));
    assert_eq!(writer.into_iter().count(), 1);
}
