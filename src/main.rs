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
};

fn main() {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;

    let busy = PinDriver::input(peripherals.pins.gpio10, Pull::Floating).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio9).unwrap();
    let rst = PinDriver::output(peripherals.pins.gpio8).unwrap();

    let sdo = peripherals.pins.gpio0;
    let sclk = peripherals.pins.gpio1;
    let cs = Some(peripherals.pins.gpio2);
    let sdi: Option<AnyInputPin> = None;

    let spi_config = DriverConfig::new();
    let spi_driver = SpiDriver::new(spi, sclk, sdo, sdi, &spi_config).unwrap();
    let spi_driver_config = Config::default();
    let mut spi_device = SpiDeviceDriver::new(spi_driver, cs, &spi_driver_config).unwrap();

    let mut epd = Epd2in13::new(&mut spi_device, busy, dc, rst, &mut Delay::new(1), None).unwrap();

    let mut display = Display2in13::default();
    display.set_rotation(DisplayRotation::Rotate90);
    let _ = Rectangle::new(Point::new(0, 0), Size::new(250, 122))
        .into_styled(PrimitiveStyle::with_fill(Color::White))
        .draw(&mut display);

    let style = MonoTextStyle::new(&FONT_6X10, Color::Black);
    let _ = Text::new("Hello World", Point::new(0, 50), style).draw(&mut display);

    let mut delay = Delay::new(10);
    epd.update_frame(&mut spi_device, display.buffer(), &mut delay)
        .unwrap();
    epd.display_frame(&mut spi_device, &mut delay).unwrap();

    epd.sleep(&mut spi_device, &mut delay).unwrap();
}
