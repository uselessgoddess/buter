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
        let mut buf = Buter::new();
        buf.extend_one(DropWaker);

        buf.iter().for_each(|_| {});
    }

    unsafe {
        assert_eq!(DROPS, 1);
    }
}

#[test]
fn drop_hard_type() {
    {
        let mut buf = Buter::new();
        buf.extend_one(String::new());

        buf.iter().for_each(|_| {});
    }
}
