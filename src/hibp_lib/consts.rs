pub const USER_AGENT: &str = concat!(
    "Rust ",
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION")
);
pub const HIBP_ROOT: &str = "https://api.pwnedpasswords.com/range/";
pub const BEGIN: u32 = 0;
pub const END: u32 = 0xFFFFF;
pub const LENGTH: u32 = END - BEGIN + 1;
