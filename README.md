# Treadlers 🏃‍♂️

A Rust library for controlling Treadly treadmills via Bluetooth Low Energy.

## Overview

Treadlers provides a complete, safe, and easy-to-use interface for controlling Treadly treadmills through Bluetooth Low Energy.

**Why this library exists**: Treadly shut down as a company, leaving their treadmills as "dead bricks" when their backend services went offline. This library was created by reverse-engineering the official Treadly Android application (version 1.1.8) to restore functionality and enable custom control applications and integrations.

The entire communication protocol was analyzed and documented, including BLE service discovery, message formats, authentication flows, and safety systems.

## Features

- 🔗 **Full BLE Protocol Support** - Complete implementation of Treadly's communication protocol
- ⚡ **Async/Await** - Built on Tokio for efficient async operations
- 🛡️ **Safety First** - Emergency stop functionality and comprehensive error handling
- 🎯 **Type Safe** - Strongly typed API preventing invalid operations
- 📊 **Status Monitoring** - Real-time device status and telemetry
- 🔐 **Authentication** - Support for device authentication when required
- 🧪 **Well Tested** - Comprehensive test suite

## Quick Start

```rust
use treadlers::{TreadlyDevice, SpeedUnit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover and connect to a Treadly device
    let mut treadmill = TreadlyDevice::connect_first().await?;
    
    // Power on the treadmill
    treadmill.power_on().await?;
    
    // Set speed to 5 km/h
    treadmill.set_speed(5.0, SpeedUnit::Kilometers).await?;
    
    // Monitor status
    let status = treadmill.get_status().await?;
    println!("Current speed: {:.1} km/h", status.speed.current);
    
    // Emergency stop if needed
    treadmill.emergency_stop().await?;
    
    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
treadlers = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Documentation

- [API Documentation](https://docs.rs/treadlers)
- [Protocol Documentation](./protocol.md) - Detailed protocol analysis
- [Examples](./examples/) - Usage examples

## Safety Warning

⚠️ **Important Safety Notice**: This library controls physical exercise equipment. Always ensure:

- Emergency stop functionality is properly implemented
- Physical safety measures are in place
- Users understand how to safely operate the equipment
- Proper error handling is implemented in your application

## Platform Support

- **Linux** ✅ Full support via BlueZ
- **macOS** ✅ Full support via Core Bluetooth  
- **Windows** ✅ Full support via WinRT Bluetooth LE APIs

## Examples

See the [examples](./examples/) directory for complete working examples:

- `basic_control.rs` - Basic treadmill control
- `status_monitor.rs` - Real-time status monitoring
- `emergency_stop.rs` - Emergency stop handling

## Development

```bash
# Clone the repository
git clone https://github.com/yourusername/treadlers
cd treadlers

# Run tests
cargo test

# Run examples
cargo run --example basic_control
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).

## Disclaimer

This library is not affiliated with or endorsed by Treadly. Use at your own risk. The protocol was reverse-engineered for educational and personal use purposes after the company ceased operations, leaving devices non-functional without their backend services.
