## const
```rust
const fn map(char: u8) -> u8 {
    match char {
        c @ b'a'..=b'z' => c - b'a' + 0xd5,
        c @ b'A'..=b'Z' => c - b'A' + 0xbb,
        c @ b'0'..=b'9' => c - b'0' + 0xa1,
        b' ' => 0x00,
        b'\n' => 0xFE,
        0 => 0xFF,
        _ => 0x00,
    }
}

const fn pkstr<const C: usize>(src: &[u8; C]) -> [u8; C] {
    let mut out: [u8; C] = [0; C];
    let mut index = 0;
    while index < C {
        out[index] = map(src[index]);
        index += 1;
    }
    out
}
const PK_STR: &[u8] = &pkstr(b"TOAST\0");
```
