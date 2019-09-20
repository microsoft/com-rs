extern crate trybuild;

#[test]
fn test_com_interface() {
    let t = trybuild::TestCases::new();
    t.compile_fail("com_interface/*.rs");
}
