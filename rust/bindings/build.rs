use std::env;
use std::path::PathBuf;

fn main() {
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base_path = env::current_dir()
        .unwrap()
        .join("../..")
        .canonicalize()
        .unwrap();
    let include_path = base_path.join("include");
    let include_path = include_path.to_str().unwrap();
    let builder = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(["-I/usr/arm-none-eabi/include", "-iquote", include_path])
        .clang_args(["--target=arm-none-eabi", "-mthumb", "-march=armv4t"])
        .allowlist_file(".*/list_menu.h")
        .allowlist_file(".*/pokemon.h")
        .allowlist_file(".*/battle.h")
        .allowlist_file(".*/item.h")
        .allowlist_file(".*/item_icon.h")
        .allowlist_file(".*/item_menu.h")
        .allowlist_file(".*/party_menu.h")
        .allowlist_file(".*/trainer_pokemon_sprites.h")
        .allowlist_file(".*/pokemon_summary_screen.h")
        .allowlist_file(".*/task.h")
        .allowlist_file(".*/malloc.h")
        .allowlist_file(".*/window.h")
        .allowlist_file(".*/text_window.h")
        .allowlist_file(".*/main.h")
        .allowlist_file(".*/text.h")
        .allowlist_file(".*/menu.h")
        .allowlist_file(".*/menu_helpers.h")
        .allowlist_file(".*/bg.h")
        .allowlist_file(".*/sprite.h")
        .allowlist_file(".*/palette.h")
        .allowlist_file(".*/decompress.h")
        .allowlist_file(".*/syscall.h")
        .allowlist_file(".*/isagbprint.h")
        .allowlist_file(".*/gpu_regs.h")
        .allowlist_file(".*/gba/.*.h")
        .allowlist_item("BATTLE_TYPE_.*")
        .allowlist_item("gMessageBox_Gfx")
        .allowlist_item("gLastViewedMonIndex")
        .allowlist_item("gTypesInfo")
        .opaque_type("PokemonSubstruct3")
        .rustified_enum("PokemonSummaryScreenMode")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core();

    let bindings = builder.clone().generate().expect("bindings");
    bindings
        .write_to_file(output_path.join("bindings.rs"))
        .expect("Writing to file");
}
