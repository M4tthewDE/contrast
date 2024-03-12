pub fn from_bytes(bytes: &[u8]) -> String {
    let hash: String = bytes.iter().fold(String::new(), |mut acc, b| {
        use std::fmt::Write; // Make sure to import `Write` at the top of your file
        write!(acc, "{:02x}", b).expect("Failed to write");
        acc
    });

    hash
}
