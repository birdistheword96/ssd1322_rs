#![no_std]
#![no_main]

use defmt::*;
use ssd1322_embassy as _;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};

use embedded_graphics::{
    prelude::*,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Gray4,
    text::{Alignment, Text},
};
use heapless::String;

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::{spi, Config};
use embassy_stm32::time::mhz;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use ssd1322_rs::{self, calculate_buffer_size, Frame, Orientation, SSD1322};
use static_cell::StaticCell;

use assign_resources::assign_resources;
use embassy_stm32::peripherals;


// Update these for your device!
assign_resources! {
    screen: ScreenResources {
        spi: SPI1,
        sck: PA5,
        mosi: PD7,
        miso: PA6,
        cs: PA4,
        reset: PB1,
        pwr: PB2,
        dc: PD6,
        dma_tx: DMA1_CH2,
    }
}

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 64;
const BUF_SIZE: usize = calculate_buffer_size(SCREEN_WIDTH, SCREEN_HEIGHT);

static FRAME_A: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();
static FRAME_B: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();

pub static NEXT_FRAME: Signal<ThreadModeRawMutex, &'static mut Frame<BUF_SIZE>> = Signal::new();
pub static READY_FRAME: Signal<ThreadModeRawMutex, &'static mut Frame<BUF_SIZE>> = Signal::new();

pub fn init_display_buffers() {
    let frame_a = FRAME_A.init(Default::default());
    NEXT_FRAME.signal(frame_a); //Set this ready to go immediately

    let frame_b = FRAME_B.init(Default::default());
    READY_FRAME.signal(frame_b); //Set this ready to go immediately
}

#[embassy_executor::task]
pub async fn render_task(screen: ScreenResources) {
    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(10);

    // let spi = spi::Spi::new_blocking(p.SPI3, p.PB3, p.PB5, p.PB4, spi_config);
    let spi_p = spi::Spi::new_txonly(
        screen.spi,
        screen.sck,
        screen.mosi,
        screen.dma_tx,
        spi_config,
    );
    let reset = Output::new(screen.reset, Level::Low, Speed::Low);
    let scr_power = Output::new(screen.pwr, Level::Low, Speed::Low);
    let data_command_pin = Output::new(screen.dc, Level::Low, Speed::Medium);
    let cs_pin = Output::new(screen.cs, Level::Low, Speed::Medium);
    let spi_dev = ExclusiveDevice::new_no_delay(spi_p, cs_pin).unwrap();

    let mut display = SSD1322::new(
        spi_dev,
        data_command_pin,
        reset,
        scr_power,
        Default::default(),
    );
    display.init_default(&mut Delay).await.unwrap();
    display
        .set_orientation(Orientation::Inverted)
        .await
        .unwrap();
    let mut frame = READY_FRAME.wait().await;
    loop {
        NEXT_FRAME.signal(frame);
        frame = READY_FRAME.wait().await;
        match display.flush_frame(&frame).await {
            Ok(_) => (),
            Err(_e) => error!("Failed to update screen"),
        }
    }
}




#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    /////////////////////////////////////////////////////////////////////
    // IMPORTANT NOTE: Update your configuration to match your board,  //
    // specifcically the PWR configuration before running.             //
    // The STM32H745 has known issues which can result in a 'bricked'  //
    // board if the wrong power configuration is written.              //
    // If you write the wrong power config, use the boot0 pin and      //
    // STMCubeProgrammer to wipe the registers under reset.            //
    let p = embassy_stm32::init(Config::default());                    //
    /////////////////////////////////////////////////////////////////////
    info!("Initialised Clocks");
    let r = split_resources!(p);

    init_display_buffers();

    spawner.must_spawn(render_task(r.screen));

    let mut ticker = Ticker::every(Duration::from_millis(15));

    let mut counter: u32 = 0;
    // Create a new character style
    let style = MonoTextStyle::new(&FONT_10X20, Gray4::WHITE);
    loop {
        let frame = NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        let mut buffer: String<32> = String::try_from("Hello from\nthe SSD1322\n").unwrap();
        let c: String<8> = counter.try_into().unwrap();
        buffer.push_str(c.as_str()).unwrap();
        Text::with_alignment(&buffer, Point::new(128, 12), style, Alignment::Center)
            .draw(frame)
            .unwrap();

        READY_FRAME.signal(frame);
        counter += 1;

        ticker.next().await;
    }
}
