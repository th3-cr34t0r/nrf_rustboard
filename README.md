RUSTBOARD

Rust based firmware for split keyboards

About:
Initially started as an ESP32 project, being build for fun and learning embedded rust.

Features:
- Supporting bluetooth
- Supporting split keyboards
- Layers
- Combos

Current bugs:
- Unable to remember paired devices

How to compile:
cargo build --release --features central / peripheral

To build uf2 firmware:
cargo make uf2 --release

will generate 2 .uf2 file, one peripheral one central

TODO:
- Central connection to be improved - (kinda improved it, need to turn on the central split, then the peripheral in order to connect correctly)
- Introduce macros feature
- Share central battery level with peripheral, show the lower value to the connected device
- Introduce sleep
- ~~Enter bootloader more easily~~ - bootloader is entered when key row:0, col:0 is held and released after 5s
- ~~Introduce combos feature~~ - done 
- ~~Solder battery and a power-switch~~ - done 
- ~~Battery level readings (saadc) feature~~ - done
- ~~Introduce a debounce feature for the matrix (sometimes with the current approach, some keys are registered 2 times)~~ - done
