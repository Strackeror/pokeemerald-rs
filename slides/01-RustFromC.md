## Calling Rust From C

### Cargo.toml
```toml
[package]
name = "pokeemerald_rs"

[lib]
crate-type = ["staticlib"]
```
### lib.rs
```rs
#[unsafe(no_mangle)]
extern "C" fn my_rust_function(a: u32, b: u32) -> u32 {
    a * 3 + b
}
```

### func.c
`gcc ... -lpokeemerald_rs func.c`
```c
uint32_t my_rust_function(uint32_t a, uint32_t b);
int func() {
    printf("%d", my_rust_function(10, 5));
}
```


---
