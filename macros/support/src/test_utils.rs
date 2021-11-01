pub(crate) mod rustfmt;

pub(crate) fn is_verbose_testing() -> bool {
    std::env::var("COM_RS_TEST_VERBOSE")
        .map(|value| matches!(value.as_str(), "1" | "true"))
        .unwrap_or_default()
}
