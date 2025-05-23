pub mod aes;
pub mod aes_msgpack;

// aes config for the japan server
pub const JAPAN_KEY: &[u8; 16] = b"g2fcC0ZczN9MTJ61";
pub const JAPAN_IV: &[u8; 16] = b"msx3IV0i9XE5uYZ1";

// aes config for the global server
pub const GLOBAL_KEY: &[u8; 16] = &[
    0xdf, 0x38, 0x42, 0x14, 0xb2, 0x9a, 0x3a, 0xdf, 0xbf, 0x1b, 0xd9, 0xee, 0x5b, 0x16, 0xf8, 0x84,
];
pub const GLOBAL_IV: &[u8; 16] = &[
    0x7e, 0x85, 0x6c, 0x90, 0x79, 0x87, 0xf8, 0xae, 0xc6, 0xaf, 0xc0, 0xc5, 0x47, 0x38, 0xfc, 0x7e,
];
