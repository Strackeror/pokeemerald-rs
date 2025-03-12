use alloc::boxed::Box;
use alloc::vec;
use core::any::TypeId;
use core::ops::AsyncFn;

use bindings::charmap::Pkstr;
use bindings::future::{Executor, sleep};
use bindings::graphics::{self, SpriteSheet, Window, *};
use bindings::input::Button;
use bindings::pokeemerald::*;
use bindings::resources::{AllocBuf, Buffer, CompressedResource};
use bindings::{include_res_lz, input, mgba_print, mgba_warn, pkstr, resources};

static EXECUTOR: Executor = Executor::new();

#[unsafe(no_mangle)]
extern "C" fn InitPresentation() {
    let fut = Box::new(presentation());

    unsafe { SetMainCallback2(Some(main_cb)) }
    EXECUTOR.set(fut);
}

extern "C" fn main_cb() {
    EXECUTOR.poll();
    unsafe {
        AnimateSprites();
        BuildOamBuffer();
        DoScheduledBgTilemapCopiesToVram();
        UpdatePaletteFade();
    }
}

extern "C" fn vblank_cb() {
    unsafe {
        LoadOam();
        ProcessSpriteCopyRequests();
        TransferPlttBuffer();
        ChangeBgX(3, 64, BG_COORD_ADD as _);
        ChangeBgY(3, 64, BG_COORD_ADD as _);
    }
}

async fn clear_ui() {
    unsafe {
        SetVBlankHBlankCallbacksToNull();
        ResetVramOamAndBgCntRegs();
        ClearScheduledBgCopiesToVram();
        sleep(1).await;

        ResetPaletteFade();
        sleep(1).await;

        ResetSpriteData();
        sleep(1).await;

        FreeAllSpritePalettes();
        sleep(1).await;

        Window::clear_all();
        sleep(1).await;

        ResetBgsAndClearDma3BusyFlags(0);
    }
}

include_res_lz!(TILESET, "../graphics/party_menu_full/tiles.4bpp");
include_res_lz!(PAL, "../graphics/party_menu_full/tiles.gbapal");
include_res_lz!(SCROLL_BG_MAP, "../graphics/party_menu_full/bg.bin");

struct Context<'a> {
    bg: BgHandle<'a>,
    fg: BgHandle<'a>,
    msg_box: TilesetHandle,
    border_gfx: TilesetHandle,
}

async fn create_msg_window(
    context: &Context<'_>,
    rect: impl Into<Rect<u8>>,
    offset: u16,
) -> Window {
    let window = Window::create(
        context.fg,
        rect.into(),
        context.msg_box.palette,
        0x80 + offset,
    );
    sleep(1).await;
    window.fill(1);
    sleep(1).await;
    window.draw_border(context.border_gfx);
    window.put_tilemap();
    window.copy_to_vram();
    sleep(1).await;
    window
}

async fn wait_a_button() {
    while !Button::A.pressed() {
        sleep(1).await
    }
}
async fn transition_slide() {
    graphics::fade_palette(PaletteMask::ALL, 5, 16, 0, 0).await;
    wait_a_button().await;
    graphics::fade_palette(PaletteMask::ALL, 5, 0, 16, 0).await;
}

fn font() -> Font {
    Font::new(FONT_SMALL as u8)
}

fn bigfont() -> Font {
    Font {
        fg_color: 4,
        shadow_color: 5,
        ..Font::new(FONT_NORMAL as u8)
    }
}

fn poke_sprite(species: u16, pos: impl Into<Vec2D<i16>>, priority: u8) -> PokemonSpritePic {
    poke_sprite_n(species, pos, priority, 0)
}

fn poke_sprite_n(
    species: u16,
    pos: impl Into<Vec2D<i16>>,
    priority: u8,
    index: u8,
) -> PokemonSpritePic {
    let mut sprite = PokemonSpritePic::new_by_index(species, index);
    sprite.handle().set_priority(priority);
    sprite.handle().set_pos(pos.into());
    sprite
}

fn print_text(window: &Window, font: Font, pos: impl Into<Vec2D<u8>>, text: &Pkstr) {
    window.print_text(text, pos.into(), font);
}

async fn slide_intro(context: &Context<'_>) {
    let window = create_msg_window(context, (10, 8, 11, 2), 0).await;
    print_text(&window, bigfont(), (0, 0), pkstr!(b"Rust in {PKMN}"));
    let _sprite = poke_sprite(98, (120, 45), 3);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_summary(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Table of contents");
    let text = pkstr!(
        b"
- Platform
- Context
- Rust and C
        - Basics
        - Abstractions
    "
    );
    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (0, 8), text);
    let _sprite = poke_sprite(6, (200, 32), 1);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_pokeemerald(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"{PKMN} Emerald Version");
    let text = pkstr!(
        b"Released in 2004 in Japan, 2005 in Europe.
Final \"mainline\" GBA {PKMN}."
    );
    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (5, 14), text);

    let text = pkstr!(
        b"Source recreations of various
games, usually in C
or assembly"
    );
    print_text(&window, bigfont(), (0, 50), pkstr!(b"Decomps"));
    print_text(&window, font(), (5, 64), text);

    let _sprite = poke_sprite(384, (200, 80), 1);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_gba(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;
    print_text(&window, bigfont(), (0, 0), pkstr!(b"The Gameboy Advance"));

    let text = pkstr!(
        b"
- Released in 2001
- ARM7TDMI CPU, 16.78Mhz, with thumb mode
- 32Kb of fast RAM, 256Kb of slower RAM
- 240x160 resolutions
- Max ROM size: 32Mb
    "
    );
    print_text(&window, font(), (0, 8), text);
    let _sprite = poke_sprite(137, (200, 32), 1);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_gba_sprites(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;
    let title = pkstr!(b"Palettes, Sprites, and Backgrounds"); 
    let text = pkstr!(
        b"
- Everything based on 8x8 pixel tiles
- 32 Palettes of 16 colors
- Up to 128 Sprites
- 4 Backgrounds constructed from tilesets
"
    );
    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (0, 3), text);
    sleep(1).await;

    let _sprite = poke_sprite(1, (40, 100), 1);
    sleep(1).await;
    let _sprite = poke_sprite_n(4, (120, 100), 1, 1);
    sleep(1).await;
    let _sprite = poke_sprite_n(7, (200, 100), 1, 2);
    sleep(1).await;

    transition_slide().await;
}

async fn slide_why(context: &Context<'_>, ican: bool) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Why ?");
    let text = pkstr!(
        b"
- Writing C is annoying
- Rust is a lot more powerful
- Fun context to try weird things in Rust
    - Is only game
- If it works it works
"
    );
    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (0, 3), text);
    let _sprite = poke_sprite(99, (200, 135), 1);

    if ican {
        print_text(&window, font(), (0, 80), pkstr!(b"- Because I can !"));
    }

    transition_slide().await;
    window.clear_with_border();
}

async fn slide_why_1(context: &Context<'_>) {
    slide_why(context, false).await;
}
async fn slide_why_2(context: &Context<'_>) {
    slide_why(context, true).await;
}

async fn slide_rust_c_call(context: &Context<'_>) {
    let window = create_msg_window(context, (8, 8, 13, 4), 0).await;

    let text = pkstr!(
        b"Calling Rust from C
And C from Rust"
    );
    print_text(&window, bigfont(), (0, 0), text);
    let _sprite = poke_sprite(82, (200, 80), 2);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_unsafe_abstractions(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 4, 26, 9), 0).await;

    let title = pkstr!(b"Unsafe abstractions");
    let text = pkstr!(
        b"
Every interaction with C code is unsafe.
So we build safe abstractions to have 
less unsafe code.
This is fraught with peril.
"
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    let _sprite = poke_sprite(871, (210, 80), 2);
    transition_slide().await;
    window.clear_with_border();
}

async fn slide_panic_handlers(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Panic handlers and alloc");
    let text = pkstr!(
        b"
panic handlers are pretty mandatory.
Has to never end so loops infinitely.

pokeemerald does have an allocator, so we
implement it as global allocator and get
all the collections !
"
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);

    sleep(1).await;
    let _sprite = poke_sprite(361, (210, 40), 1);

    transition_slide().await;
}

async fn slide_charmap(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Charmap and const");
    let text = pkstr!(
        b"
pokeemerald strings are weird !
They have a special character mapping
and they're 0xFF terminated !

Handling that at runtime is a waste,
so we use const evaluation to do it all
at compile time.
    "
    );
    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    sleep(1).await;
    let _sprite = poke_sprite(1028, (210, 40), 1);

    transition_slide().await;
}

async fn slide_async(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Async for state machines");
    let text = pkstr!(
        b"
pokeemerald uses switch-case-based state 
machines to handle logic over
multiple frames.

With async rust and a -very- simple executor,
we can handle that very cleanly !
    "
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    sleep(1).await;
    let _sprite = poke_sprite(600, (195, 120), 1);
    transition_slide().await;
}

async fn slide_phantom_data(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Zero sized lifetime wrappers");
    let text = pkstr!(
        b"
We can use phantomdata to force the compiler
to keep track of a lifetime, without storing
a reference.

Useful if the engine takes ownership of a
resource or pointer.
    "
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    sleep(1).await;
    let _sprite = poke_sprite(93, (190, 120), 1);
    transition_slide().await;
}

async fn slide_annoyances(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Problems");
    let text = pkstr!(
        b"
- as _ conversions everywhere
- mutable globals everywhere
- You still get hard to find bugs
- The stack is -very- small
    "
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    sleep(1).await;
    let _sprite = poke_sprite(474, (190, 120), 1);
    transition_slide().await;
}

async fn slide_conclusion(context: &Context<'_>) {
    let window = create_msg_window(context, (2, 2, 26, 16), 0).await;

    let title = pkstr!(b"Conclusion");
    let text = pkstr!(
        b"
- Rust is a -bit- overkill
- You have to -really- understand
   how the C part works.
- Learned a lot about the C ABI
   and Rust's unsafe pitfalls
    "
    );

    print_text(&window, bigfont(), (0, 0), title);
    print_text(&window, font(), (2, 3), text);
    sleep(1).await;
    let _sprite = poke_sprite(637, (190, 120), 1);
    transition_slide().await;
}

async fn slide_thank_you(context: &Context<'_>) {
    let window = create_msg_window(context, (8, 7, 14, 6), 0).await;

    let text = pkstr!(
        b"That's it !
Thank you for your
attention !"
    );
    print_text(&window, bigfont(), (0, 0), text);
    let _sprite = poke_sprite(202, (200, 80), 2);
    transition_slide().await;
    window.clear_with_border();
}

macro_rules! auto_dispatch {
    ($index:expr, $context:expr, $($ident:ident,)+) => {
        #[allow(clippy::all)]
        #[allow(non_camel_case_types)]
        {
            mod case {
                pub enum Case {
                    $($ident),*
                }
            }
            const LIST: &[case::Case] = &[$(case::Case::$ident),*];
            match LIST[$index] {
                $(
                case::Case::$ident => $ident($context).await,
                )*
            }
        }
    };
}

async fn slide_n(context: &Context<'_>, n: usize) -> bool {
    auto_dispatch!(
        n,
        context,
        slide_intro,
        slide_summary,
        slide_pokeemerald,
        slide_gba,
        slide_gba_sprites,
        slide_why_1,
        slide_why_2,
        slide_rust_c_call,
        slide_panic_handlers,
        slide_unsafe_abstractions,
        slide_charmap,
        slide_async,
        slide_phantom_data,
        slide_annoyances,
        slide_conclusion,
        slide_thank_you, // END
    );
    false
}

async fn presentation() {
    graphics::fade_palette(PaletteMask::ALL, 5, 0, 16, 0).await;
    clear_ui().await;
    set_gpu_registers(&[(REG_OFFSET_DISPCNT, &[DISPCNT_OBJ_ON, DISPCNT_OBJ_1D_MAP])]);

    let palettes: [BgPalette; 6] = load_bg_palettes(0, &PAL.load().get());
    let tileset: AllocBuf<TileBitmap4bpp> = TILESET.load();
    let tileset = Tileset {
        char_base: 1,
        offset: 0,
        tiles: &tileset,
        palette: palettes[0],
    };
    sleep(1).await;

    let bg_map: AllocBuf<Tile4bpp> = SCROLL_BG_MAP.load();
    let bg_size = bg_map.size_bytes();
    let bg_map = Tilemap {
        map: 0,
        buffer: &bg_map,
    };
    sleep(1).await;

    let bg = Background::load(BackgroundIndex::Background3, 3, tileset, bg_map).await;
    let bg = bg.handle();
    bg.set_pos(0, 0);
    bg.copy_tilemap_to_vram();
    bg.show();

    let map = Tilemap {
        map: 2,
        buffer: AllocBuf::new(vec![0u8; bg_size].into_boxed_slice()),
    };
    let fg = Background::load(BackgroundIndex::Background2, 2, tileset, map).await;
    let fg = fg.handle();
    fg.show();

    let msg_box = load_msg_box_gfx(fg, 0x20, 14);
    let border_gfx = load_user_window_gfx(fg, 0x50, 15);

    let context = Context {
        bg,
        fg,
        msg_box,
        border_gfx,
    };

    unsafe { SetVBlankCallback(Some(vblank_cb)) };
    for i in 0.. {
        slide_n(&context, i).await;
    }

    loop {
        sleep(1).await
    }
}
