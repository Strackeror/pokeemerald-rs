use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{BitOr, Deref};

use super::{data, sleep};
use crate::pokeemerald::{self, *};
use crate::resources::{LoadedResource, MayOwn, Resource, StaticWrapper};

pub fn set_gpu_registers(list: &[(u32, &[u32])]) {
    for (offset, flags) in list {
        let flag = flags.iter().fold(0u32, BitOr::bitor);
        unsafe { SetGpuReg(*offset as _, flag as _) };
    }
}

pub struct Tileset<'a> {
    char_base: u8,
    offset: u16,
    buffer: &'a LoadedResource,
    loaded: bool,
}

impl<'a> Tileset<'a> {
    pub const fn new(char_base: u8, offset: u16, buffer: &'a LoadedResource) -> Self {
        Tileset {
            char_base,
            offset,
            buffer,
            loaded: false,
        }
    }

    fn load(&mut self, bg_index: u8) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        unsafe {
            LoadBgTiles(
                bg_index as _,
                self.buffer.as_ptr(),
                self.buffer.len() as _,
                self.offset,
            );
        }
    }
}

pub struct Palette<'a> {
    pub buffer: MayOwn<'a, Resource>,
    pub index: usize,
}

enum PaletteType {
    Bg,
    Obj,
}

impl<'a> Palette<'a> {
    fn load(&self, palette_type: PaletteType) {
        unsafe {
            let offset = match palette_type {
                PaletteType::Bg => BG_PLTT_OFFSET as usize + self.index * 16,
                PaletteType::Obj => OBJ_PLTT_OFFSET as usize + self.index * 16,
            };
            match *self.buffer {
                Resource::Compressed { len, data } => {
                    LoadCompressedPalette(data.as_ptr().cast(), offset as _, len as _)
                }
                Resource::Direct(items) => {
                    LoadPalette(items.as_ptr().cast(), offset as _, items.len() as _);
                }
            }
        }
    }
}

pub struct Tilemap<'a> {
    pub map: u16,
    pub buffer: &'a LoadedResource,
}

pub struct Background<'a> {
    bg_index: u8,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Background<'a> {
    pub async fn load(
        index: u8,
        priority: u8,
        tileset: &mut Tileset<'a>,
        tilemap: &mut Tilemap<'a>,
        palette: &mut Palette<'a>,
    ) -> Background<'a> {
        let mut template: BgTemplate = BgTemplate::default();
        template.set_bg(index as _);
        template.set_charBaseIndex(tileset.char_base as _);
        template.set_mapBaseIndex(tilemap.map);
        template.set_baseTile(tileset.offset as _);
        template.set_paletteMode(0);
        template.set_priority(priority as _);
        template.set_screenSize(0);

        unsafe { InitBgFromTemplate(&raw const template) };
        unsafe {
            SetBgTilemapBuffer(index as _, tilemap.buffer.as_ptr());
            LoadBgTilemap(
                index as _,
                tilemap.buffer.as_ptr(),
                tilemap.buffer.len() as _,
                0,
            );
        }
        sleep(1).await;

        tileset.load(index);
        sleep(1).await;

        palette.load(PaletteType::Bg);
        sleep(1).await;

        Background {
            bg_index: index,
            _phantom_data: PhantomData,
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowBg(self.bg_index as _);
        }
    }

    pub fn set_pos(&self, x: u8, y: u8) {
        unsafe {
            ChangeBgX(self.bg_index as _, BG_COORD_SET as _, x);
            ChangeBgY(self.bg_index as _, BG_COORD_SET as _, y);
        }
    }

    pub fn fill(&self, rect: Rect<u8>, tile_index: u16, palette: u8) {
        unsafe {
            let bg = self.bg_index as u32;
            FillBgTilemapBufferRect(
                bg,
                tile_index,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                palette,
            );
        }
    }
}

pub struct SpriteHandle {
    sprite_index: u16,
}

#[unsafe(link_section = ".ewram")]
static G_SPRITES: StaticWrapper<pokeemerald::Sprite> =
    StaticWrapper::new_from_arr(&raw mut gSprites);
impl SpriteHandle {
    pub fn set_pos(&mut self, x: i16, y: i16) {
        let mut sprite = G_SPRITES.index_mut(self.sprite_index as usize);
        sprite.x = x;
        sprite.y = y;
    }
    pub fn set_palette(&mut self, palette: u16) {
        let mut sprite = G_SPRITES.index_mut(self.sprite_index as usize);
        sprite.oam.set_paletteNum(palette);
    }
    pub fn set_invisible(&mut self, invisible: bool) {
        let mut sprite = G_SPRITES.index_mut(self.sprite_index as usize);
        sprite.set_invisible(invisible.then_some(1).unwrap_or(0));
    }
    pub fn animate(&mut self) {
        unsafe {
            let mut sprite = G_SPRITES.index_mut(self.sprite_index as usize);
            AnimateSprite(&raw mut *sprite);
        }
    }
    pub fn request_copy(&self) {
        unsafe {
            let sprite = G_SPRITES.index_mut(self.sprite_index as usize);
            RequestSpriteFrameImageCopy(0, sprite.oam.tileNum(), sprite.images);
        }
    }
}

pub struct SpriteAnims {
    anims: *const *const AnimCmd,
    affine_anims: *const *const AffineAnimCmd,
}
unsafe impl Sync for SpriteAnims {}
pub static DUMMY_SPRITE_ANIMS: SpriteAnims = unsafe {
    SpriteAnims {
        anims: gDummySpriteAnimTable.as_ptr(),
        affine_anims: gDummySpriteAffineAnimTable.as_ptr(),
    }
};

pub struct SpriteImage {
    pub resource: LoadedResource,
    pub size: u32,
}

pub struct Sprite<'a> {
    #[expect(unused)]
    palette: MayOwn<'a, Palette<'a>>,
    #[expect(unused)]
    image: MayOwn<'a, SpriteImage>,
    #[expect(unused)]
    frame: Box<SpriteFrameImage>,
    #[expect(unused)]
    anims: &'a SpriteAnims,
    sprite: SpriteHandle,
}

impl<'a> Sprite<'a> {
    pub async fn load(
        anims: &'a SpriteAnims,
        image: impl Into<MayOwn<'a, SpriteImage>>,
        palette: impl Into<MayOwn<'a, Palette<'a>>>,
    ) -> Self {
        let palette: MayOwn<'a, Palette<'a>> = palette.into();
        let image: MayOwn<'a, SpriteImage> = image.into();

        let mut template = SpriteTemplate::default();
        template.affineAnims = anims.affine_anims;
        template.anims = anims.anims;
        template.callback = Some(SpriteCallbackDummy);

        let mut frame = SpriteFrameImage::default();
        frame.data = image.resource.as_ptr();
        frame.relativeFrames = 0;
        frame.size = (image.resource.len()) as u16;
        let frame = Box::new(frame);
        template.images = &raw const *frame;

        let mut oam = OamData::default();
        oam.set_size((image.size >> 2) & 0b11);
        oam.set_shape(image.size & 0b11);

        template.oam = &raw const oam;
        template.tileTag = TAG_NONE as _;
        template.paletteTag = TAG_NONE as _;
        palette.load(PaletteType::Obj);

        let sprite_index = unsafe { CreateSprite(&raw const template, 0, 0, 0) };
        let mut sprite = SpriteHandle {
            sprite_index: sprite_index as _,
        };
        sprite.set_palette(palette.index as u16);
        sprite.request_copy();
        Sprite {
            palette,
            image,
            anims,
            sprite,
            frame,
        }
    }

    pub fn handle(&mut self) -> &mut SpriteHandle {
        &mut self.sprite
    }
}
impl<'a> Drop for Sprite<'a> {
    fn drop(&mut self) {
        unsafe {
            DestroySpriteAndFreeResources(
                &raw mut *&mut *G_SPRITES.index_mut(self.sprite.sprite_index as _),
            );
        }
    }
}

pub struct PokemonSpritePic {
    sprite: SpriteHandle,
}

impl PokemonSpritePic {
    pub fn new(poke: &data::Pokemon, slot: u8) -> PokemonSpritePic {
        let species = poke.species();
        let personality = poke.personality();
        let shiny = poke.shiny();
        unsafe {
            let sprite_index = CreateMonPicSprite_Affine(
                species,
                shiny as _,
                personality,
                MON_PIC_AFFINE_FRONT as _,
                0,
                0,
                slot,
                TAG_NONE as _,
            );
            PokemonSpritePic {
                sprite: SpriteHandle { sprite_index },
            }
        }
    }

    pub fn sprite(&mut self) -> &mut SpriteHandle {
        &mut self.sprite
    }
}

impl Drop for PokemonSpritePic {
    fn drop(&mut self) {
        unsafe {
            FreeAndDestroyMonPicSprite(self.sprite.sprite_index);
        }
    }
}

pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T> Rect<T> {
    pub fn new(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Rect<u8> {
    pub fn tile_to_pixel(self) -> Rect<u16> {
        Rect {
            x: self.x as u16 * 8,
            y: self.y as u16 * 8,
            width: self.width as u16 * 8,
            height: self.height as u16 * 8,
        }
    }
}

pub struct WindowHandle {
    index: u32,
}

impl WindowHandle {
    pub fn blit_bitmap(&self, pixels: &[u8], rect: Rect<u16>) {
        unsafe {
            BlitBitmapToWindow(
                self.index as _,
                pixels.as_ptr(),
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            )
        };
    }

    pub fn fill(&self, fill: u8) {
        unsafe { FillWindowPixelBuffer(self.index as _, fill) };
    }

    pub fn display(&self) {
        unsafe {
            CopyWindowToVram(self.index, COPYWIN_FULL);
            PutWindowTilemap(self.index);
        }
    }
    pub fn copy_tilemap(&self, tileset: &Tileset, tilemap: &Tilemap, rect: Rect<u8>) {
        const SIZE_OF_TILE: usize = 32;
        let mut buffer = Vec::with_capacity(rect.width as usize * rect.height as usize * 32);
        for y in 0..rect.height {
            for x in 0..rect.width {
                let tilemap_index = x + rect.width * y;
                let tile_index = tilemap.buffer.buffer()[tilemap_index as usize];
                let pixel_offset = tile_index as usize * SIZE_OF_TILE;
                let tile_data = &tileset.buffer.buffer()[pixel_offset..pixel_offset + SIZE_OF_TILE];
                buffer.extend_from_slice(tile_data);
            }
        }
        self.blit_bitmap(&buffer, rect.tile_to_pixel());
    }
}

pub struct Window<'a> {
    _own: MayOwn<'a, Palette<'a>>,
    handle: WindowHandle,
}

impl<'a> Deref for Window<'_> {
    type Target = WindowHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a> Window<'a> {
    pub fn create(
        bg: u8,
        rect: Rect<u8>,
        palette: impl Into<MayOwn<'a, Palette<'a>>>,
        base_block: u16,
    ) -> Window<'a> {
        let palette = palette.into();
        let mut window_template = WindowTemplate::default();
        window_template.baseBlock = base_block;
        window_template.bg = bg;
        window_template.paletteNum = palette.index as _;
        window_template.tilemapLeft = rect.x;
        window_template.tilemapTop = rect.y;
        window_template.width = rect.width;
        window_template.height = rect.height;
        let index = unsafe { AddWindow(&raw const window_template) };
        let handle = WindowHandle { index };
        Window {
            _own: palette,
            handle,
        }
    }

    pub fn create_with_tilemap(
        bg: u8,
        rect: Rect<u8>,
        palette: impl Into<MayOwn<'a, Palette<'a>>>,
        tilemap: &Tilemap<'a>,
        base_block: u16,
    ) -> Window<'a> {
        let palette = palette.into();
        let mut window_template = WindowTemplate::default();
        window_template.baseBlock = base_block;
        window_template.bg = bg;
        window_template.paletteNum = palette.index as _;
        window_template.tilemapLeft = rect.x;
        window_template.tilemapTop = rect.y;
        window_template.width = rect.width;
        window_template.height = rect.height;

        let index = unsafe { AddWindowWithoutTileMap(&raw const window_template) };
        unsafe { SetWindowAttribute(index as _, WINDOW_TILE_DATA, tilemap.buffer.as_ptr() as u32) };

        let handle = WindowHandle {
            index: index as u32,
        };
        Window {
            _own: palette,
            handle,
        }
    }
}

impl Drop for Window<'_> {
    fn drop(&mut self) {
        unsafe { RemoveWindow(self.handle.index as _) };
    }
}
