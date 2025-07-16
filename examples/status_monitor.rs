use std::time::Duration;
use tokio::time::{interval, Instant};
use tracing::{error, info, warn};
use treadlers::{Result, TreadlyDevice};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("📊 Treadlers Status Monitor Example");
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

    info!("🔍 Starting status monitoring...");
    info!("Press Ctrl+C to stop monitoring");

    // Set up monitoring interval
    let mut monitor_interval = interval(Duration::from_secs(2));
    let start_time = Instant::now();
    let mut last_distance = 0.0;
    let mut max_speed = 0.0;

    loop {
        monitor_interval.tick().await;

        // Get current status
        let status = treadmill.get_status().await;

        // Update tracking variables
        if status.speed.current > max_speed {
            max_speed = status.speed.current;
        }

        let distance_delta = status.distance - last_distance;
        last_distance = status.distance;

        // Display status
        let elapsed = start_time.elapsed();
        let minutes = elapsed.as_secs() / 60;
        let seconds = elapsed.as_secs() % 60;

        println!("\n📊 Status Update ({minutes:02}:{seconds:02})");
        println!("┌─────────────────────────────────────────┐");
        println!(
            "│ Speed: {:6.1} {} (Target: {:5.1})     │",
            status.speed.current, status.speed.unit, status.speed.target
        );
        println!("│ Mode:  {:20}           │", format!("{}", status.mode));
        println!(
            "│ Power: {:20}           │",
            if status.power_on { "ON" } else { "OFF" }
        );
        println!("│ Distance: {:8.2} km               │", status.distance);
        println!("│ Steps: {:12}                   │", status.steps);
        println!("│ Temperature: {:6.1}°C              │", status.temperature);
        println!(
            "│ Emergency: {:18}        │",
            format!("{}", status.emergency_stop)
        );
        println!(
            "│ Handrail: {:19}        │",
            if status.handrail_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!("└─────────────────────────────────────────┘");

        // Display session statistics
        if status.session_active || status.distance > 0.0 {
            println!("📈 Session Statistics:");
            println!("  Max Speed: {:.1} {}", max_speed, status.speed.unit);
            println!("  Total Distance: {:.2} km", status.distance);
            println!("  Total Steps: {}", status.steps);
            if distance_delta > 0.0 {
                println!("  Pace: +{distance_delta:.3} km this update");
            }
        }

        // Display warnings if needed
        if matches!(status.emergency_stop, treadlers::EmergencyStopState::Active) {
            println!("⚠️  WARNING: Emergency stop is ACTIVE!");
        }

        if status.temperature > 40.0 {
            println!(
                "🌡️  WARNING: High temperature detected: {:.1}°C",
                status.temperature
            );
        }

        if !status.power_on {
            println!("💤 Device is powered off");
        }

        // Check connection
        if !treadmill.is_connected().await {
            warn!("❌ Device disconnected");
            break;
        }
    }

    // Disconnect
    info!("🔌 Disconnecting...");
    if let Err(e) = treadmill.disconnect().await {
        error!("❌ Failed to disconnect: {}", e);
    } else {
        info!("✅ Disconnected successfully");
    }

    println!("\n📊 Final Session Summary:");
    println!(
        "  Duration: {:02}:{:02}",
        start_time.elapsed().as_secs() / 60,
        start_time.elapsed().as_secs() % 60
    );
    println!("  Max Speed: {max_speed:.1} km/h");
    println!("  Total Distance: {last_distance:.2} km");

    info!("🎉 Status monitoring completed!");
    Ok(())
}
