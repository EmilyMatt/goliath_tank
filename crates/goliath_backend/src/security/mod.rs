mod certification_generator;

pub use certification_generator::generate_certification_and_keys;

// This is so you don't get confused and use the wrong one
pub struct GoliathCert(pub Vec<u8>);
pub struct GoliathPKey(pub Vec<u8>);
