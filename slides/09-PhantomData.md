## PhantomData
### sprite.rs
```rust
struct Sprite<'a> {
    index: usize,
    _own: PhantomData<&'a ()>,
}

impl Sprite<'_> {
    fn create<'a>(buffer: &'a [u8]) -> Sprite<'a> {
        let index = unsafe { CreateSprite(buffer.as_ptr()) };
        Sprite {
            index,
            _own: PhantomData,
        }
    }
}

impl Drop for Sprite<'_> {
    fn drop(&mut self) {
        unsafe {
            DestroySprite(&raw mut gSprites[self.index]);
        }
    }
}
```

### incorrect.rs
```rust
fn toast<'a>() -> Sprite<'a> {
    let sprite_data: Vec<u8> = Vec::new();
    Sprite::create(&sprite_data) // ERROR: cannot return value referencing local variable
}
```