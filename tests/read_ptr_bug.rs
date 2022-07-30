use buter::Buter;

#[test]
fn read() {
    let buter = Buter::with_capacity(1);

    let mut writer = buter.writer();
    writer.extend(Some(String::from("234")));

    let s1 = writer.into_iter().next().unwrap();

    let mut writer = buter.writer();
    writer.extend(Some(String::from("432")));

    let s2 = writer.into_iter().next().unwrap();

    assert_ne!(s1, s2);
}
