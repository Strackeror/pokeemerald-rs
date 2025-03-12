## Unsafe
### sprite.c
```c
extern struct Sprite gSprites[64];
u32 CreateSprite(...);
void DestroySprite(struct Sprite *sprite);
```
### sprite.rs
```rust
struct Sprite {
    index: usize
}

impl Sprite {
    fn create(...) -> Self {
        let index = unsafe { CreateSprite(...) };
        Sprite { index }
    }

    fn set_pos(&self, x: u8, y: u8) {
        unsafe {
            gSprites[self.index].x = x;
            gSprites[self.index].x = y;
        }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        unsafe {
            DestroySprite(&raw mut gSprites[self.index]);
        }
    }
}
```

### call.rs
```rust
fn show_sprite() {
    let sprite = Sprite::create(...); // Sprite appears
    while !Button::A.is_pressed() { }
} // drop called, Sprite disappears
```