#[inline]
pub fn is_f32_enough(number: f64) -> bool {
    ((number as f32) as f64) != number
}
