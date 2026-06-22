/// Rounds a floating-point number to two decimal places and returns it as a string.
pub fn round_float(value: f32) -> String {
    format!("{:.2}", value)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
