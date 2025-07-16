use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use treadlers::{Result, SpeedUnit, TreadlyDevice};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🏃‍♂️ Treadlers Basic Control Example");
    info!("Searching for Treadly devices...");

    // Connect to the first available Treadly device
    let treadmill = match TreadlyDevice::connect_first().await {
        Ok(device) => {
            info!("✅ Connected to: {}", device.device_info().name);
            device
        }
        Err(e) => {
            error!("❌ Failed to connect to device: {}", e);
            return Err(e);
        }
    };

    // Display initial status
    let status = treadmill.get_status().await;
    info!("📊 Initial Status:");
    info!("  Mode: {}", status.mode);
    info!("  Power: {}", if status.power_on { "ON" } else { "OFF" });
    info!("  Speed: {:.1} {}", status.speed.current, status.speed.unit);
    info!("  Emergency Stop: {}", status.emergency_stop);

    // Power on the treadmill
    info!("🔌 Powering on treadmill...");
    if let Err(e) = treadmill.power_on().await {
        error!("❌ Failed to power on: {}", e);
        return Err(e);
    }
    info!("✅ Treadmill powered on");

    // Wait a moment for the device to be ready
    sleep(Duration::from_secs(2)).await;

    // Set speed to 3 km/h
    info!("⚡ Setting speed to 3.0 km/h...");
    if let Err(e) = treadmill.set_speed(3.0, SpeedUnit::Kilometers).await {
        error!("❌ Failed to set speed: {}", e);
        return Err(e);
    }
    info!("✅ Speed set to 3.0 km/h");

    // Wait and then increase speed
    sleep(Duration::from_secs(5)).await;

    info!("📈 Increasing speed...");
    for _ in 0..3 {
        if let Err(e) = treadmill.speed_up().await {
            error!("❌ Failed to increase speed: {}", e);
            break;
        }
        info!("✅ Speed increased");
        sleep(Duration::from_secs(2)).await;
    }

    // Check current status
    let status = treadmill.get_status().await;
    info!("📊 Current Status:");
    info!("  Speed: {:.1} {}", status.speed.current, status.speed.unit);
    info!("  Mode: {}", status.mode);

    // Gradually decrease speed
    info!("📉 Decreasing speed...");
    loop {
        let status = treadmill.get_status().await;
        if status.speed.current <= 0.1 {
            break;
        }

        if let Err(e) = treadmill.speed_down().await {
            error!("❌ Failed to decrease speed: {}", e);
            break;
        }
        info!(
            "✅ Speed decreased to {:.1} {}",
            status.speed.current, status.speed.unit
        );
        sleep(Duration::from_secs(2)).await;
    }

    // Final status check
    let status = treadmill.get_status().await;
    info!("📊 Final Status:");
    info!("  Speed: {:.1} {}", status.speed.current, status.speed.unit);
    info!("  Distance: {:.2} km", status.distance);
    info!("  Steps: {}", status.steps);
    info!("  Temperature: {:.1}°C", status.temperature);

    // Disconnect
    info!("🔌 Disconnecting...");
    treadmill.disconnect().await?;
    info!("✅ Disconnected successfully");

    info!("🎉 Basic control example completed!");
    Ok(())
}
