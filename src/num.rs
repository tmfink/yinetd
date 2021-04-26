use std::cmp;

pub fn clamp(x: i32, min: i32, max: i32) -> i32 {
    cmp::min(cmp::max(min, x), max)
}
