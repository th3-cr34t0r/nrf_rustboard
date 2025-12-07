How to compile:
cargo build --release --features central / peripheral

To build uf2 firmware:
cargo make uf2 --release

will generate 2 .uf2 file, one peripheral one central

TODO:
- Central connection to be improved (kinda improved it, need to turn on the central split, then the peripheral in order to connect correctly)
- ~~Enter bootloader more easily~~ - bootloader is entered when key row:0, col:0 is held and released after 5s
- Introduce combos feature 
- Introduce macros feature 
- Solder battery and a power-switch
- Battery level readings (saadc) feature
