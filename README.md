# ssd1322_rs

## Table of Contents
- [ssd1322\_rs](#ssd1322_rs)
  - [Table of Contents](#table-of-contents)
  - [About ](#about-)
  - [Features ](#features-)
  - [Usage ](#usage-)
  - [Examples](#examples)
  - [Acknowledgements](#acknowledgements)
  - [Pull requests](#pull-requests)

## About <a name = "about"></a>

`ssd1322_rs` is an asynchronous Rust driver for the SSD1322 OLED screen, specifically designed for embedded systems. It provides an easy-to-use, non-blocking interface for interacting with the display, making it ideal for use with DMA.

The `ssd1322_rs` driver was created to fill a gap in the ecosystem for an asynchronous driver for the SSD1322 OLED screen. This project takes inspiration from the existing [ssd1322 crate][ssd1322], but changes the design from synchronous to asynchronous. It also switches the design philosophy from using an iterator pattern (which minimises memory usage) to using a fixed static buffer (when the `frame` feature is enabled), which allows DMA to perform the heavy lifting and minimises CPU time spent transferring data.

For reference, the screen I created this crate for is a 256x64 pixel screen. As the SSD1322 uses 4bpp, this means that the total bytes to be transferred is:

$$
Total Bytes = 128×64=8192 
$$

$$
Total bits  = 8192 × (8bits/byte)=65536
$$

Running the SPI bus at 10Mhz, this results in:

$$
Time_s = \frac{Total Bits}{SPI clock speed} = \frac{65535 \ bits}{10,000,000 \ bits/second} = 0.0065536 \ seconds
$$

Which is therefore:

$$
TimeToTransfer = 0.0065536s ×1000  = 6.5536 \ ms
$$

The performance hit of trasnfering without DMA (6.5ms blocking) was too much for our application, hence the creation of this crate, which focuses on performance throughput at the expense of memory efficiency (still no alloc or heap memory though!).

## Features <a name = "features"></a>

- Asynchronous API: Utilizes Rust's async/await syntax for non-blocking operations.
- Supports SPI communication protocol only (Feel free to make a pull request if you want to add another comms method!).
- Simple and complex API: Default setup takes care of the usual configuration, with command options to manually configure the screen.
- No Standard Library: Suitable for `#![no_std]` environments.
- No alloc
- Minimal dependecies: Only relies on the embedded hal crates (with optional support for `embedded_graphics` which can be disabled with `default-features=false`)
- Optional `frame` buffer (supporting `embedded_graphics` to simplify handling screen to a fixed-size bufffer)

## Usage <a name = "usage"></a>
The following code is an exmaple of how to set up the screen. This code uses embassy on an STM32H745 as an example, but this crate is executor agnostic, so any async runtime could be used. Any chip that supports SPI is also compatible.

```rust
    use ssd1322_rs::{Orientation, SSD1322};

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(10);

    // Create an SPI instance, we only need TX for this library
    let spi_p = spi::Spi::new_txonly(
        screen.spi,
        screen.sck,
        screen.mosi,
        screen.dma_tx,
        spi_config,
    );

    // Reset pin required to hard reset and initilaise the screen
    let reset = Output::new(screen.reset, Level::Low, Speed::Low);
    //Power on the screen
    let scr_power = Output::new(screen.pwr, Level::Low, Speed::Low);
    // Control command/data mode
    let data_command_pin = Output::new(screen.dc, Level::Low, Speed::Medium);

    // Chip select
    let cs_pin = Output::new(screen.cs, Level::Low, Speed::Medium);

    // Create an Exclusive device from the from the embedded_hal, for SPI busses with only one other device on.
    let spi_dev = ExclusiveDevice::new_no_delay(spi_p, cs_pin).unwrap();

    // Create a display handle
    let mut display = SSD1322::new(
        spi_dev,
        data_command_pin,
        reset,
        scr_power,
        Default::default(),
    );

    // Initialise the display. This calls the reset procedue and then sends the neccesary commands to set up the display
    display.init_default(&mut Delay).await.unwrap();
```

To use the display, you can either use it with or without frabe support. For this example, we will use the built-in default frame support to flush a frame to the display:

```rust
use ssd1322_rs::{calculate_buffer_size, Frame, Orientation, SSD1322};
use static_cell::StaticCell;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 64;
const BUF_SIZE: usize = calculate_buffer_size(SCREEN_WIDTH, SCREEN_HEIGHT);

static FRAME_A: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();

let frame = FRAME_A.init(Default::default());
//Draw some stuff into the buffer
//embedded_graphics::Text::with_alignment("hello world", Point::new(128, 12), MonoTextStyle::new(&FONT_10X20, Gray4::WHITE), Alignment::Center).draw(frame).unwrap();
display.flush_frame(&frame).await.ok();
```

## Examples

Check out the [examples](examples/) folder for practical demonstrations of how to use the `async-ssd1322` driver in your projects.

## Acknowledgements
Shout out to the various crates that inspired this one, and to the embedded rust community as a whole:

[The original ssd1322 crate][ssd1322] - This was a great reference for how to use the commands on the SSD1322.

[st7735-embassy][st7735-embassy] - The main structure (and frame buffer) was inspired heavily by the design architecture of this crate

[embassy][embassy] - The embassy project is an amazing contribution to the embedded rust ecosystem, and is the de-facto standard on for how to run async code on an embedded device.

## Pull requests
Pull requests welcome for any added features, functionality, or bug fixes!

[embassy]:[https://github.com/embassy-rs]
[ssd1322]: [https://github.com/edarc/ssd1322]
[st7735-embassy]: [https://github.com/kalkyl/st7735-embassy]
