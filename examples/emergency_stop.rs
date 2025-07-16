use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use treadlers::{EmergencyStopState, Result, SpeedUnit, TreadlyDevice};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚨 Treadlers Emergency Stop Example");
    info!("This example demonstrates emergency stop functionality");

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

    // Check initial emergency stop state
    let status = treadmill.get_status().await;
    info!("📊 Initial emergency stop state: {}", status.emergency_stop);

    // If emergency stop is active, reset it first
    if matches!(
        status.emergency_stop,
        EmergencyStopState::Active | EmergencyStopState::ResetRequired
    ) {
        warn!("⚠️ Emergency stop is currently active, resetting...");
        if let Err(e) = treadmill.reset_emergency_stop().await {
            error!("❌ Failed to reset emergency stop: {}", e);
            return Err(e);
        }
        info!("✅ Emergency stop reset");
        sleep(Duration::from_secs(2)).await;
    }

    // Power on the treadmill
    info!("🔌 Powering on treadmill...");
    if let Err(e) = treadmill.power_on().await {
        error!("❌ Failed to power on: {}", e);
        return Err(e);
    }
    sleep(Duration::from_secs(2)).await;

    // Set a moderate speed
    info!("⚡ Setting speed to 5.0 km/h...");
    if let Err(e) = treadmill.set_speed(5.0, SpeedUnit::Kilometers).await {
        error!("❌ Failed to set speed: {}", e);
        return Err(e);
    }

    // Wait for the treadmill to reach target speed
    info!("⏳ Waiting for treadmill to reach target speed...");
    for i in 1..=10 {
        sleep(Duration::from_secs(1)).await;
        let status = treadmill.get_status().await;
        info!("  {}s - Current speed: {:.1} km/h", i, status.speed.current);

        if (status.speed.current - 5.0).abs() < 0.5 {
            break;
        }
    }

    let status = treadmill.get_status().await;
    info!("📊 Current status before emergency stop:");
    info!("  Speed: {:.1} km/h", status.speed.current);
    info!("  Mode: {}", status.mode);
    info!("  Emergency Stop: {}", status.emergency_stop);

    // Demonstrate emergency stop
    warn!("🚨 ACTIVATING EMERGENCY STOP in 3 seconds...");
    sleep(Duration::from_secs(1)).await;
    warn!("🚨 EMERGENCY STOP in 2 seconds...");
    sleep(Duration::from_secs(1)).await;
    warn!("🚨 EMERGENCY STOP in 1 second...");
    sleep(Duration::from_secs(1)).await;

    // Trigger emergency stop
    warn!("🚨 EMERGENCY STOP ACTIVATED!");
    if let Err(e) = treadmill.emergency_stop().await {
        error!("❌ Failed to activate emergency stop: {}", e);
        return Err(e);
    }

    // Monitor the emergency stop effect
    info!("📊 Monitoring emergency stop effect...");
    for i in 1..=10 {
        sleep(Duration::from_secs(1)).await;
        let status = treadmill.get_status().await;

        info!("  {}s after emergency stop:", i);
        info!("    Speed: {:.1} km/h", status.speed.current);
        info!("    Emergency Stop: {}", status.emergency_stop);

        if status.speed.current < 0.1 {
            info!("✅ Treadmill has stopped");
            break;
        }
    }

    // Final status check
    let status = treadmill.get_status().await;
    info!("📊 Final status after emergency stop:");
    info!("  Speed: {:.1} km/h", status.speed.current);
    info!("  Mode: {}", status.mode);
    info!("  Emergency Stop: {}", status.emergency_stop);

    // Demonstrate emergency stop reset
    if matches!(
        status.emergency_stop,
        EmergencyStopState::Active | EmergencyStopState::ResetRequired
    ) {
        info!("🔄 Demonstrating emergency stop reset...");
        sleep(Duration::from_secs(2)).await;

        if let Err(e) = treadmill.reset_emergency_stop().await {
            error!("❌ Failed to reset emergency stop: {}", e);
        } else {
            info!("✅ Emergency stop reset successfully");

            // Check status after reset
            sleep(Duration::from_secs(1)).await;
            let status = treadmill.get_status().await;
            info!("📊 Status after reset:");
            info!("  Emergency Stop: {}", status.emergency_stop);
        }
    }

    // Test handrail safety (if supported)
    info!("🛡️ Testing handrail safety settings...");

    let status = treadmill.get_status().await;
    let current_handrail_state = status.handrail_enabled;
    info!(
        "  Current handrail state: {}",
        if current_handrail_state {
            "Enabled"
        } else {
            "Disabled"
        }
    );

    // Toggle handrail setting
    info!("  Toggling handrail setting...");
    if let Err(e) = treadmill
        .set_handrail_enabled(!current_handrail_state)
        .await
    {
        warn!("⚠️ Failed to change handrail setting: {}", e);
    } else {
        sleep(Duration::from_secs(1)).await;
        let status = treadmill.get_status().await;
        info!(
            "  New handrail state: {}",
            if status.handrail_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );

        // Restore original setting
        if let Err(e) = treadmill.set_handrail_enabled(current_handrail_state).await {
            warn!("⚠️ Failed to restore handrail setting: {}", e);
        } else {
            info!("  Handrail setting restored");
        }
    }

    // Disconnect
    info!("🔌 Disconnecting...");
    treadmill.disconnect().await?;
    info!("✅ Disconnected successfully");

    info!("🎉 Emergency stop example completed!");
    info!("Key takeaways:");
    info!("  - Emergency stop immediately stops the treadmill");
    info!("  - Emergency stop state must be reset before normal operation");
    info!("  - Handrail safety can be configured");
    info!("  - Always implement proper error handling for safety commands");

    Ok(())
}
