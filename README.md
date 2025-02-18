# pokeemerald-rs
This repo is an example of integrating rust code within pokeemerald
The interesting part is in the `rust` folder, there are 2 crates:
- The bindings crates contains the actual generated c bindings, plus a few safe wrappers for different concepts of pokeemerald
- The pokeemerald_rs crates contains an example of a party screen replacement
    - This example contains some graphics from [RavePossum's BW summary screen](https://github.com/ravepossum/pokeemerald-expansion/tree/bw_summary_screen_expansion)

To compile, in addition to the standard pokeemerald dependencies, you'll need rust nightly and libclang/clang installed.
The rust lib is handled by the makefile system, so if you have the correct setup, `make` should be enough to build the rom
