#![no_std]
#![warn(missing_docs)]

//! This is an SSD1322 Crate

pub mod instruction;
use crate::instruction::{Command, CommandError, CommandData, consts::{BUF_COL_MAX, PIXEL_ROW_MAX}};
use core::convert::Infallible;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiDevice;

/// Calculates the buffer size required for the 4bpp SSD1322 display based on its width and height.
///
/// This function is evaluated at compile time and returns the size of the buffer needed
/// to hold the pixel data for a display of the given dimensions.
///
/// # Parameters
///
/// - `width`: The width of the display in pixels.
/// - `height`: The height of the display in pixels.
///
/// # Returns
///
/// The buffer size required to hold the pixel data for a display of the specified width and height.
pub const fn calculate_buffer_size(width: usize, height: usize) -> usize {
    (width * height) / 2
}

/// Async SSD1322 OLED display driver.
///
/// This struct provides an interface for controlling the SSD1322 OLED display
/// using SPI communication. It supports asynchronous operations and allows 
/// customization of the data/command pin, reset pin, and power-on pin.
///
/// # Type Parameters
///
/// - `SPI`: The SPI device used for communication with the display.
/// - `DC`: The data/command pin, used to switch between sending data and commands.
/// - `RST`: The reset pin, used to reset the display.
/// - `PO`: The power on pin, used to power on the display.
///
/// # Constraints
///
/// - `SPI`: Must implement the `SpiDevice` trait.
/// - `DC`, `RST`, `PO`: Must implement the `OutputPin` trait with `Error = Infallible`.
pub struct SSD1322<SPI, DC, RST, PO>
where
    SPI: SpiDevice,
    DC: OutputPin<Error = Infallible>,
    RST: OutputPin<Error = Infallible>,
    PO: OutputPin<Error = Infallible>,
{
    /// SPI device used for communication with the display.
    spi: SPI,
    /// Data/command pin, used to switch between sending data and commands.
    dc: DC,
    /// Reset pin, used to reset the display.
    rst: RST,
    /// Power on pin, used to power on the display.
    power: PO,
    /// Whether the colors are inverted (`true`) or not (`false`).
    inverted: bool,
    /// Orientation of the display.
    orientation: Orientation,
}


/// Display orientation.
///
/// This enum represents the different possible orientations supported in this library for the display.
/// The orientations determine how the image is mapped onto the display.
///
/// # Variants
///
/// - `Standard`: The standard orientation (value: 0x06). 
///   This corresponds to setting the remap first byte to 0x06 and the second byte to 0x11.
/// - `Inverted`: The inverted orientation (value: 0x14).
///   This corresponds to an inverted display mapping.
#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    /// Standard orientation (0x06).
    /// Set remap first byte, second byte should be 0x11.
    Standard = 0x06,
    /// Inverted orientation (0x14).
    Inverted = 0x14,
}

/// Optional configuration structure to invertthe colour or screen orientation
pub struct Config {
    inverted_colour: bool,
    orientation: Orientation,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inverted_colour: false,
            orientation: Orientation::Standard,
        }
    }
}

impl<SPI, DC, RST, PO, E> SSD1322<SPI, DC, RST, PO>
where
    SPI: SpiDevice<Error = E>,
    DC: OutputPin<Error = Infallible>,
    RST: OutputPin<Error = Infallible>,
    PO: OutputPin<Error = Infallible>,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(spi: SPI, dc: DC, rst: RST, power:PO, config: Config) -> Self {
        Self {
            spi,
            dc,
            rst,
            power,
            inverted: config.inverted_colour,
            orientation: config.orientation,
        }
    }

    /// Runs commands to initialize the display in the default configuration for this library. In most use cases, this should
    /// be all that is needed to start and set-up the device.
    /// 
    /// # Non Default Configuration
    ///
    /// If you do not want to use the default configuration, you can set up the display with the following pattern:
    ///
    /// ```
    /// # use ssd1322_rs::{SSD1322, CommandData, DelayNs, Error};
    /// # async fn example_usage<D>(display: &mut SSD1322<SPI, DC, RST, PO>, delay: &mut D) -> Result<(), Error<E>>
    /// # where
    /// #     D: DelayNs,
    /// # {
    ///     display.hard_reset(&mut delay).await?; // Must be called first
    ///
    ///     display.write_command(Command::SetStartLine(0).prepare()?).await?;
    ///     display.write_command(Command::SetDisplayOffset(0).prepare()?).await?;
    ///     /*Add other Display commands here */
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// - `delay`: A mutable reference to an implementation of the `DelayNs` trait.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the initialization succeeds, or an `Error` if it fails.
    pub async fn init_default<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayNs,

    {
        use crate::instruction::*;
        self.hard_reset(delay).await?;
        let dc = &mut self.dc;
        let inverted = self.inverted;


        let commands = [
            Command::SetCommandLock(false).prepare()?,
            Command::SetSleepMode(true).prepare()?,
            Command::SetRemapping(IncrementAxis::Horizontal,
                ColumnRemap::Forward,
                NibbleRemap::Forward,
                ComScanDirection::RowZeroLast,
                ComLayout::DualProgressive,
            ).prepare()?,
            Command::SetStartLine(0).prepare()?,
            Command::SetDisplayOffset(0).prepare()?,
            Command::SetDisplayMode(
                {
                    if inverted {DisplayMode::Inverse} else {DisplayMode::Normal}
                }).prepare()?,
            Command::FunctionSelect(FunctionSelection::InternalVDD).prepare()?,
            Command::SetPhaseLengths(5, 15).prepare()?,
            Command::SetClockFoscDivset(10, 1).prepare()?,
            Command::SetDisplayEnhancements(true, true).prepare()?,
            Command::SetSecondPrechargePeriod(8).prepare()?,
            Command::SetDefaultGrayScaleTable.prepare()?,
            Command::SetPreChargeVoltage(31).prepare()?,
            Command::SetComDeselectVoltage(7).prepare()?,
            Command::SetContrastCurrent(0x3C).prepare()?,
            Command::SetMasterContrast(0xA).prepare()?,
            Command::SetMuxRatio(0x3F).prepare()?,
            Command::DisablePartialDisplay.prepare()?,
            // Don't bother setting DisplayB enhancements,
            Command::SetSleepMode(false).prepare()?,
        ];

        for CommandData {
            cmd,
            data,
            len
        } in commands
        {
            dc.set_low().ok();
            let mut tx_data = [0_u8; 1];
            tx_data.copy_from_slice(&[cmd as u8]);
            self.spi.write(&tx_data).await.map_err(Error::Comm)?;
            if !data.is_empty() {
                dc.set_high().ok();
                let mut buf = [0_u8; 2];
                buf.copy_from_slice(&data);
                self.spi
                    .write(&buf[..len])
                    .await
                    .map_err(Error::Comm)?;
            }
        }

        self.set_orientation(self.orientation).await?;
        Ok(())
    }

    /// Performs a hard reset power-on sequence as described in section 8.9 of the SSD1322 Manual.
    ///
    /// The power-on sequence is as follows:
    ///
    /// 1. Power ON VCI and VDDIO.
    /// 2. After VCI and VDDIO become stable, wait at least 1ms (t0) for internal VDD to stabilize.
    /// 3. Set the RES# pin LOW (logic low) for at least 100µs (t1) and then HIGH (logic high).
    /// 4. After setting the RES# pin LOW, wait for at least 100µs (t2), then power ON VCC.
    ///
    /// The datasheet suggests sending command AFh to turn the display ON, and SEG/COM will be ON after 200ms.
    /// However, instead of turning on the display immediately, we configure the screen before turning it on.
    ///
    /// # Parameters
    ///
    /// - `delay`: A mutable reference to an implementation of the `DelayNs` trait, used to introduce delays in the sequence.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the reset sequence completes successfully, or an `Error` if it fails.
    pub async fn hard_reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayNs,
    {
        delay.delay_ms(1).await;
        self.rst.set_low().map_err(Error::Pin)?;
        delay.delay_ms(1).await;
        self.rst.set_high().map_err(Error::Pin)?;
        delay.delay_ms(1).await;
        self.power.set_high().map_err(Error::Pin)?;
        delay.delay_ms(1).await;
        Ok(())
    }

    /// Configures the orientation of the display.
    ///
    /// This function sets the orientation of the display to either standard or inverted. 
    /// It updates the internal orientation state and sends the appropriate command to the display 
    /// to adjust the remapping configuration.
    ///
    /// # Parameters
    ///
    /// - `orientation`: The desired orientation for the display. Can be either `Orientation::Standard` or `Orientation::Inverted`.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the orientation is successfully set, or an `Error` if the operation fails.
    ///
    /// ```
    /// # use your_crate::{ssd1322_rs, Orientation, Error};
    /// # async fn example_usage(display: &mut SSD1322<SPI, DC, RST, PO>) -> Result<(), Error<E>> {
    /// display.set_orientation(Orientation::Inverted).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error<E>> {
        use crate::instruction::*;
        self.orientation = orientation;
        if orientation == Orientation::Inverted{
            self.write_command(&Command::SetRemapping(
                IncrementAxis::Horizontal,
                ColumnRemap::Reverse,
                NibbleRemap::Forward,
                ComScanDirection::RowZeroFirst,
                ComLayout::DualProgressive,
            ).prepare()?).await?;
        } else {
            self.write_command(&Command::SetRemapping(
                IncrementAxis::Horizontal,
                ColumnRemap::Forward,
                NibbleRemap::Forward,
                ComScanDirection::RowZeroLast,
                ComLayout::DualProgressive,
            ).prepare()?).await?;
        }
        Ok(())
    }

    /// Sends a command to the SSD1322 display.
    ///
    /// This function writes a command to the SSD1322 display using SPI communication. 
    /// It first sets the data/command pin low to indicate that a command is being sent, 
    /// writes the command, and if there is any associated data, it sets the data/command 
    /// pin high and writes the data.
    ///
    /// # Parameters
    ///
    /// - `command`: A reference to a `CommandData` struct containing the command byte and optional data bytes.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the command is successfully written, or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `Error::Comm`: If there is a communication error during the SPI write operation.
    async fn write_command(
        &mut self,
        command: &CommandData,
    ) -> Result<(), Error<E>> {
        let dc = &mut self.dc;
        dc.set_low().ok();

        self.spi.write(&[command.cmd]).await.map_err(Error::Comm)?;

        if command.len != 0 {
            dc.set_high().ok();
            self.spi
                .write(&command.data[..command.len])
                .await
                .map_err(Error::Comm)?;
        }
        Ok(())
    }

    // Helper function to set the DC pin high ready for Data transmission
    fn start_data(&mut self) -> Result<(), Error<E>> {
        // Write command isnt in here because this isnt an async function
        self.dc.set_high().map_err(Error::Pin)
    }

    /// Sends data to the SSD1322 display.
    ///
    /// This function sends data from a buffer to the screen. It assumes the correct initialisation 
    /// commands have already been sent, and that the address window has been set/
    ///
    /// # Parameters
    ///
    /// - `data`: A reference to a buffer containing the pixel data.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the command is successfully written, or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `Error::Comm`: If there is a communication error during the SPI write operation.
    pub async fn write_data(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        self.start_data()?;
        self.spi.write(&data).await.map_err(Error::Comm)
    }

    /// Sets the address window for the display.
    ///
    /// This function defines a rectangular area (address window) on the display where subsequent drawing operations will be applied.
    /// It calculates the necessary column and row addresses based on the provided dimensions and starting coordinates.
    ///
    /// # Parameters
    ///
    /// - `start_x`: The starting x-coordinate (horizontal) of the address window.
    /// - `start_y`: The starting y-coordinate (vertical) of the address window.
    /// - `width`: The width of the address window.
    /// - `height`: The height of the address window.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the address window is successfully set, or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `Error::CommandError(CommandError::OutOfRange)`: If the calculated column or row addresses are out of the valid range.
    ///
    /// # Example
    ///
    /// ```
    /// # use ssd1322_rs::{SSD1322, Error};
    /// # async fn example_usage(display: &mut SSD1322<SPI, DC, RST, PO>) -> Result<(), Error<E>> {
    /// display.set_address_window(0, 0, 256, 64).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_address_window(
        &mut self,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
    ) -> Result<(), Error<E>> {
        let offset = (480-width+start_x) / 2;
        let column_start = offset / 4;
        let column_end = (column_start + (width / 4)) - 1;

        if column_start > BUF_COL_MAX.into() || column_end > BUF_COL_MAX.into() ||
        start_y > PIXEL_ROW_MAX.into() || height > PIXEL_ROW_MAX.into() {
         return Err(Error::CommandError(CommandError::OutOfRange))
        }

        self.write_command(&Command::SetColumnAddress(column_start as u8, column_end as u8).prepare()?).await?;
        self.write_command(&Command::SetRowAddress(start_y as u8, (height-1) as u8).prepare()?).await
    }

    /// Flushes the provided buffer to the display.
    ///
    /// This function writes the contents of the provided buffer to the display's RAM.
    /// It sends the `WriteRam` command to the display, followed by the buffer data.
    ///
    /// # Parameters
    ///
    /// - `buf`: A slice containing the data to be written to the display.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the buffer is successfully flushed to the display, or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `Error::Comm`: If there is a communication error during the SPI write operation.
    ///
    /// # Example
    ///
    /// ```
    /// # use ssd1322_rs::{SSD1322, Error};
    /// # async fn example_usage(display: &mut SSD1322<SPI, DC, RST, PO>, buffer: &[u8]) -> Result<(), Error<E>> {
    /// display.flush_buffer(buffer).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn flush_buffer(&mut self, buf: &[u8]) -> Result<(), Error<E>> {
        self.write_command(&Command::WriteRam.prepare()?).await?;
        self.start_data()?;
        self.spi.write(buf).await.map_err(Error::Comm)
    }

    /// Flushes the provided frame to the display.
    ///
    /// This function writes the contents of the provided frame buffer to the display's RAM.
    /// It sets the address window to the dimensions of the frame, sends the `WriteRam` command,
    /// and then writes the frame data to the display.
    ///
    /// > **This function is only available when the `frame` feature is enabled.**
    ///
    /// # Parameters
    ///
    /// - `frame`: A reference to a `Frame` struct containing the data to be written to the display.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the frame is successfully flushed to the display, or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `Error::Comm`: If there is a communication error during the SPI write operation.
    ///
    /// # Example
    ///
    /// ```
    /// # use ssd1322_rs::{SSD1322, Frame, Error};
    /// # async fn example_usage<const N: usize>(display: &mut SSD1322<SPI, DC, RST, PO>, frame: &Frame<N>) -> Result<(), Error<E>> {
    /// display.flush_frame(frame).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "frame")]
    pub async fn flush_frame<const N: usize>(&mut self, frame: &Frame<N>) -> Result<(), Error<E>> {
        self.set_address_window(0, 0, frame.width, frame.height)
            .await?;
        self.write_command(&Command::WriteRam.prepare()?).await?;
        self.write_data(&frame.buffer).await
    }
}

/// Error Types used within this crate
#[derive(Debug)]
pub enum Error<E = ()> {
    /// Communication error
    Comm(E),
    /// Pin setting error
    Pin(Infallible),
    ///Command Error
    CommandError(CommandError),
}

impl<E> From<CommandError> for Error<E> {
    fn from(err: CommandError) -> Self {
        Error::CommandError(err)
    }
}


/// Frame buffer module for the SSD1322 display.
///
/// This module provides the `Frame` struct, which represents a frame buffer for the SSD1322 display.
/// It includes functionality for creating, manipulating, and drawing to the frame buffer using the `embedded-graphics` traits.

#[cfg(feature = "frame")]
mod frame {
    extern crate embedded_graphics_core;
    use self::embedded_graphics_core::{
        draw_target::DrawTarget,
        pixelcolor::{
            raw::{RawData, RawU4},
            Gray4,
        },
        prelude::*,
    };


    /// A frame buffer for the SSD1322 display.
    ///
    /// The `Frame` struct contains the screen buffer and metadata such as width, height, and orientation.
    /// It implements the `DrawTarget` trait from the `embedded-graphics` crate, allowing for easy drawing operations.
    pub struct Frame<const N: usize> {
        /// The width of the frame buffer in pixels.
        pub width: u16,
        /// The height of the frame buffer in pixels.
        pub height: u16,
        /// The buffer storing the pixel data.
        pub buffer: [u8; N],
    }

    impl<const N: usize> Frame<N> {
        /// Creates a new `Frame` with the specified dimensions and buffer.
        ///
        /// # Parameters
        ///
        /// - `width`: The width of the frame buffer in pixels.
        /// - `height`: The height of the frame buffer in pixels.
        /// - `orientation`: The orientation of the display.
        /// - `buffer`: The buffer storing the pixel data.
        ///
        /// # Returns
        ///
        /// A new `Frame` instance.
        pub fn new(width: u16, height: u16, buffer: [u8; N]) -> Self {
            Self {
                width,
                height,
                buffer,
            }
        }

        /// Sets a pixel in the frame buffer to the specified color.
        ///
        /// # Parameters
        ///
        /// - `x`: The x-coordinate of the pixel.
        /// - `y`: The y-coordinate of the pixel.
        /// - `color`: The greyscale colour to set the pixel to.
        pub fn set_pixel(&mut self, x: u8, y: u8, color: Gray4) {
            let color = RawU4::from(color).into_inner();
            if x as usize >= self.width as usize || y as usize >= self.height as usize {
                return;
            }
            let idx = ((y as usize) * self.width as usize + (x as usize)) / 2;
            if idx >= self.buffer.len() {
                return;
            }

            // Extract the 4-bit color value
            let color = color & 0x0F;

            if x % 2 == 0 {
                // Set the higher 4 bits for even x coordinate
                self.buffer[idx] = (self.buffer[idx] & 0x0F) | (color << 4);
            } else {
                // Set the lower 4 bits for odd x coordinate
                self.buffer[idx] = (self.buffer[idx] & 0xF0) | color;
            }
        }
    }
    impl<const N: usize> Default for Frame<N> {
        fn default() -> Self {
            Self {
                width: 256,
                height: 64,
                buffer: [0; N],
            }
        }
    }

    impl<const N: usize> DrawTarget for Frame<N> {
        type Error = ();
        type Color = Gray4;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            let bb = self.bounding_box();
            pixels
                .into_iter()
                .filter(|Pixel(pos, _color)| bb.contains(*pos))
                .for_each(|Pixel(pos, color)| self.set_pixel(pos.x as u8, pos.y as u8, color));
            Ok(())
        }

        fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
            let c = RawU4::from(color).into_inner();
            let packed_color = (c << 4) | c; // Pack two 4-bit color values into one byte
            for i in 0..self.buffer.len() {
                self.buffer[i] = packed_color;
            }
            Ok(())
        }
    }

    impl<const N: usize> OriginDimensions for Frame<N> {
        fn size(&self) -> Size {
            Size::new(self.width as u32, self.height as u32)
        }
    }

}


#[cfg(feature = "frame")]
pub use frame::*;