#[macro_export]
macro_rules! n_bits {
    ($l:literal) => {
        !(u32::MAX << $l)
    };
}

#[macro_export]
macro_rules! clear_mask {
    ($bits:literal, $shift:literal) => {
        !(crate::n_bits!($bits) << $shift)
    };
}
