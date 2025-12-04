How to compile:
cargo build --release --features central / peripheral

To build uf2 firmware:
cargo make uf2 --release

will generate 2 .uf2 file, one peripheral one central

TODO:
- Central connection to be improved
- Enter bootloader more easily
- Introduce combos feature 
- Introduce macros feature 
- Solder battery and a power-switch
- Battery level readings (saadc) feature
