pub(crate) mod rustfmt;

pub(crate) fn is_verbose_testing() -> bool {
    if let Ok(value) = std::env::var("COM_RS_TEST_VERBOSE") {
        match value.as_str() {
            "1" | "true" => true,
            _ => false,
        }
    } else {
        false
    }
}
