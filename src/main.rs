use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::{Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use epd_waveshare::{
    color::Color,
    epd2in13_v2::{Display2in13, Epd2in13},
    prelude::{DisplayRotation, WaveshareDisplay},
};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{AnyInputPin, PinDriver, Pull},
    peripherals::Peripherals,
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver,
    },
    units::FromValueType, // Required for .MHz()
};

fn main() {
    // It's necessary to call this function to link the patches
    // for the esp-idf-sys crate
    esp_idf_svc::sys::link_patches();

    // Bind the log to the ESP-IDF log output
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi3;

    println!("Setting up GPIOs on the left side...");
    // BUSY (Input), RST and DC (Output)
    // Note: Some Waveshare boards need Pull::Up, others Pull::Floating.
    // We'll use Up to be safe.
    let busy = PinDriver::input(peripherals.pins.gpio32, Pull::Down).unwrap();
    let rst = PinDriver::output(peripherals.pins.gpio33).unwrap(); // Was 33
    let dc = PinDriver::output(peripherals.pins.gpio25).unwrap(); // Was 25

    // SPI Pins - Strictly Left Side
    let sdo = peripherals.pins.gpio14; // DIN / MOSI
    let sclk = peripherals.pins.gpio27; // CLK / SCK
    let cs = Some(peripherals.pins.gpio26); // CS
    let sdi: Option<AnyInputPin> = None; // MISO not used

    println!("Initializing SPI Driver at 2MHz...");
    let spi_config = DriverConfig::new();
    let spi_driver = SpiDriver::new(spi, sclk, sdo, sdi, &spi_config).unwrap();

    // E-Papers fail frequently if SPI speed is too high (default is often >40MHz)
    let spi_driver_config = Config::new().baudrate(2.MHz().into());
    let mut spi_device = SpiDeviceDriver::new(spi_driver, cs, &spi_driver_config).unwrap();

    let mut delay = Delay::new(100);

    println!("Initializing E-Paper Display...");
    // Increased delay for the reset cycle to 100ms
    let mut epd = Epd2in13::new(&mut spi_device, busy, dc, rst, &mut delay, None)
        .expect("e-Paper hardware init failed - check wiring!");

    println!("Creating Display Buffer...");
    let mut display = Display2in13::default();
    display.set_rotation(DisplayRotation::Rotate90);

    // Clear the buffer with White
    let _ = Rectangle::new(Point::new(0, 0), Size::new(250, 122))
        .into_styled(PrimitiveStyle::with_fill(Color::White))
        .draw(&mut display);

    // Draw Text
    let style = MonoTextStyle::new(&FONT_6X10, Color::Black);
    let _ = Text::new("Hello Rust World!", Point::new(10, 50), style).draw(&mut display);

    println!("Updating Frame (sending data)...");
    epd.update_frame(&mut spi_device, display.buffer(), &mut delay)
        .expect("Failed to send frame to EPD");

    println!("Refreshing Display (physical update)...");
    epd.display_frame(&mut spi_device, &mut delay)
        .expect("Failed to trigger display refresh");

    println!("Putting EPD to sleep...");
    epd.sleep(&mut spi_device, &mut delay).unwrap();

    println!("Done! Screen should show content now.");
}
