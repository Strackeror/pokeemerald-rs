## Calling C from Rust
### lib.c
```c
uint32_t my_c_function(uint32_t a, uint32_t b){
    return a * 3 + b;
}
```
### lib.rs
```rs
unsafe extern "C" {
    fn my_c_function(a: u32, b: u32) -> u32;
}

fn to_string() -> String {
    unsafe {
        format!("{}", my_c_function(10, 5))
    }
}
```
