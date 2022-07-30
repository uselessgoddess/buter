#![feature(extend_one)]

use buter::Buter;

static mut DROPS: usize = 0;

struct DropWaker;

impl Drop for DropWaker {
    fn drop(&mut self) {
        unsafe {
            DROPS += 1;
        }
    }
}

#[test]
fn drop_count() {
    {
        let buf = Buter::new();
        let mut writer = buf.writer();
        writer.extend_one(DropWaker);

        writer.into_iter().for_each(|_| {});
    }

    unsafe {
        assert_eq!(DROPS, 1);
    }
}

#[test]
fn drop_hard_type() {
    let buf = Buter::new();
    let mut writer = buf.writer();
    writer.extend_one(String::new());

    writer.into_iter().for_each(|_| {});
}
