# SSD1322 LCD example using STM32H7

The purpose of this example is to demonstrate how to use the library, rather than to directly run this example. The code in this example uses a pinout for a custom in-house PCB, so if you do wan't to run this, you will need to modify the pinout for your PCB. See the `assign_resources!` macro for the pin definitions:

```rust
use assign_resources::assign_resources;
use embassy_stm32::peripherals;

assign_resources! {
    screen: ScreenResources {
        spi: SPI1,  //SPI Peripheral
        sck: PA5,   //Clock
        mosi: PD7,  //Data Out
        miso: PA6,  //Data In (NOT Needed as screen only talks one way)
        cs: PA4,    // Chip Select
        reset: PB1, // SSD1322 Reset pin
        pwr: PB2,   // SSD1322 PowerOn pin
        dc: PD6,    // SSD1322 Data/Command Pin
        dma_tx: DMA1_CH2,  //DMA Channel, may be STM32 specific
    }
}
```

## Important Note: Configuration for the STM32H7

If you want to run the example on an STM32H7, like this example does, then make sure to update your config **before** running the example. 

```rust
let p = embassy_stm32::init(Config::default());
```

you may instead need somthing like:

```rust
let config = stm32::Config;
config.rcc.supply_config = SupplyConfig::DirectSMPS; // Switched Mode Power Supply
```
.

The STM32H7 's have known "Gotcha's" which can result in a 'bricked' board if the wrong power configuration is written. If you do write the wrong power config, use the boot0 pin and STMCubeProgrammer to wipe the registers under reset.


## Run example

If you still want to go ahead an run the example, then you will need to update the `.cargo/config.toml` to use your probe, and confirm the same chip type. In this repo, its set up for the `cm7` core of the `STM32H745ZIT3`

Then, you can run the example like this:

```shell
$ cargo rrb frames
```
or 
```shell
$ cargo run --release --bin frames
```


