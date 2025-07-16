#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! # Treadlers 🏃‍♂️
//!
//! A Rust library for controlling Treadly treadmills via Bluetooth Low Energy.
//!
//! This library provides a complete, safe, and easy-to-use interface for controlling
//! Treadly treadmills through Bluetooth Low Energy. The entire communication protocol
//! was reverse-engineered from the official Treadly Android application (version 1.1.8)
//! after the company shut down their backend services, leaving the treadmills
//! as "dead bricks".
//!
//! ## Reverse Engineering Details
//!
//! The protocol implementation in this library is based on detailed analysis of the
//! decompiled Android application, including:
//!
//! - **BLE Service Discovery**: Nordic UART Service (NUS) based communication
//! - **Message Protocol**: 20-byte fixed message format with command/response structure  
//! - **Authentication**: Secret key extraction and authentication flow analysis
//! - **Command Mapping**: Complete mapping of all 70+ available treadmill commands
//! - **Status Parsing**: Detailed status message structure and bit field layouts
//! - **Safety Systems**: Emergency stop mechanisms and handrail safety features
//!
//! See `protocol.md` for comprehensive documentation of the reverse-engineered protocol.
//!
//! ## Safety Warning
//!
//! ⚠️ **Important**: This library controls physical exercise equipment. Always ensure:
//! - Emergency stop functionality is properly implemented
//! - Physical safety measures are in place
//! - Users understand how to safely operate the equipment
//! - Proper error handling is implemented in your application
//!
//! ## Quick Start
//!
//! ```no_run
//! use treadlers::{TreadlyDevice, SpeedUnit};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Discover and connect to a Treadly device
//!     let mut treadmill = TreadlyDevice::connect_first().await?;
//!     
//!     // Power on the treadmill
//!     treadmill.power_on().await?;
//!     
//!     // Set speed to 5 km/h
//!     treadmill.set_speed(5.0, SpeedUnit::Kilometers).await?;
//!     
//!     // Emergency stop if needed
//!     treadmill.emergency_stop().await?;
//!     
//!     Ok(())
//! }
//! ```

/// Bluetooth Low Energy communication module
pub mod ble;
/// Main device control interface
pub mod device;
/// Error types and handling
pub mod error;
/// Protocol message structures and parsing
pub mod protocol;
/// Type definitions and data structures
pub mod types;

// Re-export the main types for convenient usage
pub use device::TreadlyDevice;
pub use error::{Result, TreadlyError};
pub use types::{
    AuthenticationStatus, ConnectionHealth, ConnectionParams, DeviceInfo, DeviceMode, DeviceStatus,
    DeviceStatusCode, EmergencyStopState, SpeedInfo, SpeedUnit, TemperatureStatus, TimeoutConfig,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Treadly BLE Service UUID discovered through reverse engineering
///
/// This UUID is based on the Nordic UART Service (NUS) and was extracted from
/// the decompiled Treadly Android app v1.1.8. The service provides the primary
/// communication channel for all treadmill commands and status updates.
pub const TREADLY_SERVICE_UUID: &str = "6E400000-B5A3-F393-E0A9-E50E24DCCA9E";

/// Treadly TX Characteristic UUID for device-to-app notifications
///
/// This characteristic is used by the treadmill to send status updates, responses,
/// and notifications back to the controlling application. Discovered through
/// analysis of the official Android app's BLE communication patterns.
pub const TREADLY_TX_CHAR_UUID: &str = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E";

/// Treadly RX Characteristic UUID for app-to-device commands
///
/// This characteristic is used to send control commands from the application to
/// the treadmill. All command messages follow the 20-byte protocol format
/// discovered through reverse engineering of the official app.
pub const TREADLY_RX_CHAR_UUID: &str = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E";

/// Treadly manufacturer ID for device discovery during BLE scanning
///
/// Value 0x59 (89 decimal) was extracted from the Android app's device discovery
/// logic. This manufacturer ID is included in the BLE advertisement data and
/// helps identify authentic Treadly devices during scanning.
pub const TREADLY_MANUFACTURER_ID: u16 = 0x59;
