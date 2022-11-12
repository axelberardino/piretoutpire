mod math;
pub use math::{distance, div_ceil, middle_point};

mod raw_array;
pub use raw_array::{
    u32_list_to_u8_array, u32_list_to_u8_array_unfailable, u32_to_u8_array, u8_array_to_u32,
    u8_array_to_u32_list,
};
