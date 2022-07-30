#[test]
#[cfg(not(miri))]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}
