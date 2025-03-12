## Async and state machines
```rust

async fn show_party_menu() {
    unsafe {
        SetVBlankHBlankCallbacksToNull();
        frame().await;
        ScanlineEffect_Stop();
        frame().await;
        /* ... */
        alloc_party_menu_bg_gfx().await;
    }
}

async fn alloc_party_menu_bg_gfx() {
    unsafe {
        LoadBgTiles();
        frame().await;
        /* ... */
        PartyPaletteBufferCopy(8);
        frame().await;
    }
}
```