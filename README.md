# E-Ink with Rust for the ESP32
Before running check if the pins align correctly with your hardware setup.

## Run & Flash the ESP32
~~~bash
cargo run --release
~~~

## Known issue
There are a few pixel which are not displayed correctly, because the crate 'epd-waveshare' hasn't set the width correctly.
Therefore the epd-waveshare crate is added as a submodule to this repository with said problem fixed.
