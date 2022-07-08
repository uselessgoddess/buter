Do you have callbacks (and other difficult for iterators) and are you tired of `.collect()` them into `Vecs`? There is a
solution.

`Buter` - buffered iterator with fluid buffer, which is used as a place for all iterations

### Usage

```rust
fn callback_work<F>(mut f: F)
where
    F: FnMut(usize),
{
    for i in 0..10_000 {
        f(i);
    }
}

use buter::Buter;

struct Worker {
    buf: Buter<usize>,
    // ...
}

impl Worker {
    fn work(&self) -> impl Iterator<Item = usize> + '_ {
        let mut writer = self.buf.writer();
        callback_work(|i| writer.extend(Some(i)));
        writer.into_iter()
    }
}
```

### Benches
This is appropriate both with large results and with small ones
```rust
// test buter                  ... bench:    14 ns/iter (+/- 5)
// test vec_push               ... bench:   212 ns/iter (+/- 130)
// test vec_push_with_capacity ... bench:    54 ns/iter (+/- 32)
```
```rust
// buter                   time:   [1.6348 ms 1.6445 ms 1.6549 ms]
// vec_push                time:   [4.4204 ms 4.4750 ms 4.5360 ms]
// vec_push_with_capacity  time:   [2.6034 ms 2.6338 ms 2.6680 ms]
```
