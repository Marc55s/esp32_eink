use embedded_graphics::{
    Drawable, mono_font::{MonoTextStyle, ascii::FONT_6X10, iso_8859_1::FONT_6X13}, prelude::{Point, Primitive, Size}, primitives::{PrimitiveStyle, Rectangle}, text::Text
};
use epd_waveshare::{
    color::Color,
    epd2in13_v2::{Display2in13, Epd2in13},
    prelude::{DisplayRotation, WaveshareDisplay},
};
use esp_idf_svc::hal::{
    delay::{Delay, Ets, FreeRtos},
    gpio::{AnyInputPin, PinDriver, Pull},
    peripherals::Peripherals,
    spi::{
        SpiDeviceDriver, SpiDriver, config::{Config, DriverConfig}
    },
};

fn main() {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;

    let busy = PinDriver::input(peripherals.pins.gpio4, Pull::Down).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio16).unwrap();
    let rst = PinDriver::output(peripherals.pins.gpio17).unwrap();

    let sclk = peripherals.pins.gpio18; // VSPI CLK
    let sdo = peripherals.pins.gpio23; // VSPI MOSI
    let cs = Some(peripherals.pins.gpio5); // VSPI CS
    let sdi: Option<AnyInputPin> = None;

    let spi_config = DriverConfig::new();
    let spi_driver = SpiDriver::new(spi, sclk, sdo, sdi, &spi_config).unwrap();
    let spi_driver_config = Config::default();
    let mut spi_device = SpiDeviceDriver::new(spi_driver, cs, &spi_driver_config).unwrap();

    let mut ets = Ets;
    let mut epd = Epd2in13::new(&mut spi_device, busy, dc, rst, &mut ets, None).unwrap();
    epd.clear_frame(&mut spi_device, &mut ets).unwrap();
    let mut display = Display2in13::default();
    display.set_rotation(DisplayRotation::Rotate90);
    let _ = Rectangle::new(Point::new(0, 0), Size::new(250, 130))
        .into_styled(PrimitiveStyle::with_fill(Color::White))
        .draw(&mut display);

    let style = MonoTextStyle::new(&FONT_6X13, Color::Black);
    let _ = Text::new("Hello World", Point::new(0, 50), style).draw(&mut display);

    // let mut delay = Delay::new(10);
    epd.update_frame(&mut spi_device, display.buffer(), &mut ets)
        .unwrap();
    epd.display_frame(&mut spi_device, &mut ets).unwrap();

    epd.sleep(&mut spi_device, &mut ets).unwrap();
    loop {
        FreeRtos::delay_ms(1000);
    }
}
