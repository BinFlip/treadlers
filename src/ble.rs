use btleplug::{
    api::{BDAddr, Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType},
    platform::{Manager, Peripheral},
};
use futures::stream::StreamExt;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    time::timeout,
};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    error::{Result, TreadlyError},
    protocol::Message,
    types::{ConnectionParams, DeviceInfo},
    TREADLY_MANUFACTURER_ID, TREADLY_RX_CHAR_UUID, TREADLY_SERVICE_UUID, TREADLY_TX_CHAR_UUID,
};

/// BLE manager for Treadly device communication
pub struct BleManager {
    manager: Manager,
    peripherals: Arc<Mutex<HashMap<BDAddr, Peripheral>>>,
    notification_sender: Option<mpsc::UnboundedSender<Message>>,
}

impl BleManager {
    /// Create a new BLE manager
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::Ble`] if the Bluetooth adapter cannot be initialized.
    pub async fn new() -> Result<Self> {
        let manager = Manager::new().await?;

        Ok(Self {
            manager,
            peripherals: Arc::new(Mutex::new(HashMap::new())),
            notification_sender: None,
        })
    }

    /// Scan for Treadly devices
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::DeviceNotFound`] if no Bluetooth adapters are available,
    /// or [`TreadlyError::Ble`] for other Bluetooth-related errors.
    pub async fn scan_for_devices(&self, params: &ConnectionParams) -> Result<Vec<DeviceInfo>> {
        info!("Starting scan for Treadly devices...");

        let adapters = self.manager.adapters().await?;
        if adapters.is_empty() {
            return Err(TreadlyError::DeviceNotFound);
        }

        let central = &adapters[0];

        let service_uuid = Uuid::parse_str(TREADLY_SERVICE_UUID)
            .map_err(|e| TreadlyError::Protocol(format!("Invalid service UUID: {e}")))?;

        let scan_filter = ScanFilter {
            services: vec![service_uuid],
        };

        central.start_scan(scan_filter).await?;

        tokio::time::sleep(Duration::from_millis(params.scan_timeout_ms)).await;

        central.stop_scan().await?;

        let peripherals = central.peripherals().await?;
        let mut devices = Vec::new();
        for peripheral in peripherals {
            if self.is_treadly_device(&peripheral).await {
                let device_info = self.extract_device_info(&peripheral).await;
                devices.push(device_info.clone());

                self.peripherals
                    .lock()
                    .await
                    .insert(peripheral.address(), peripheral);

                info!("Found Treadly device: {}", device_info.name);
            }
        }

        info!("Scan completed. Found {} Treadly device(s)", devices.len());
        Ok(devices)
    }

    /// Connect to a specific device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::DeviceNotFound`] if the device cannot be found,
    /// [`TreadlyError::Timeout`] if connection times out,
    /// [`TreadlyError::ConnectionFailed`] if connection fails,
    /// or [`TreadlyError::Protocol`] for protocol-related errors.
    pub async fn connect_to_device(
        &mut self,
        device_info: &DeviceInfo,
        params: &ConnectionParams,
    ) -> Result<TreadlyConnection> {
        info!("Connecting to device: {}", device_info.name);

        let peripherals = self.peripherals.lock().await;
        let peripheral = peripherals
            .values()
            .find(|p| {
                if let Ok(Some(properties)) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(p.properties())
                }) {
                    properties.local_name.as_ref() == Some(&device_info.name)
                } else {
                    false
                }
            })
            .ok_or_else(|| TreadlyError::DeviceNotFound)?
            .clone();

        drop(peripherals);

        let connect_future = peripheral.connect();
        timeout(Duration::from_millis(params.timeout_ms), connect_future)
            .await
            .map_err(|_| TreadlyError::Timeout {
                timeout_ms: params.timeout_ms,
            })?
            .map_err(|e| TreadlyError::ConnectionFailed(e.to_string()))?;

        peripheral.discover_services().await?;

        let service_uuid = Uuid::parse_str(TREADLY_SERVICE_UUID)
            .map_err(|e| TreadlyError::Protocol(format!("Invalid service UUID: {e}")))?;
        let tx_char_uuid = Uuid::parse_str(TREADLY_TX_CHAR_UUID)
            .map_err(|e| TreadlyError::Protocol(format!("Invalid TX characteristic UUID: {e}")))?;
        let rx_char_uuid = Uuid::parse_str(TREADLY_RX_CHAR_UUID)
            .map_err(|e| TreadlyError::Protocol(format!("Invalid RX characteristic UUID: {e}")))?;

        let services = peripheral.services();
        let service = services
            .iter()
            .find(|s| s.uuid == service_uuid)
            .ok_or_else(|| TreadlyError::Protocol("Treadly service not found".to_string()))?;

        let tx_char = service
            .characteristics
            .iter()
            .find(|c| c.uuid == tx_char_uuid)
            .ok_or_else(|| TreadlyError::Protocol("TX characteristic not found".to_string()))?
            .clone();

        let rx_char = service
            .characteristics
            .iter()
            .find(|c| c.uuid == rx_char_uuid)
            .ok_or_else(|| TreadlyError::Protocol("RX characteristic not found".to_string()))?
            .clone();

        let (notification_tx, notification_rx) = mpsc::unbounded_channel();
        self.notification_sender = Some(notification_tx.clone());

        peripheral.subscribe(&tx_char).await?;

        info!("Successfully connected to {}", device_info.name);

        Ok(TreadlyConnection {
            peripheral,
            tx_char,
            rx_char,
            notification_receiver: notification_rx,
            _notification_sender: notification_tx,
        })
    }

    /// Check if the device properties indicate a Treadly device  
    async fn is_treadly_device(&self, peripheral: &Peripheral) -> bool {
        if let Ok(Some(properties)) = peripheral.properties().await {
            if let Some(name) = &properties.local_name {
                if name.to_lowercase().contains("treadly") {
                    return true;
                }
            }

            if properties
                .manufacturer_data
                .contains_key(&TREADLY_MANUFACTURER_ID)
            {
                return true;
            }
        }

        false
    }

    /// Extract device information from BLE properties
    async fn extract_device_info(&self, peripheral: &Peripheral) -> DeviceInfo {
        if let Ok(Some(properties)) = peripheral.properties().await {
            let name = properties
                .local_name
                .clone()
                .unwrap_or_else(|| "Unknown Treadly".to_string());

            let rssi = properties.rssi.unwrap_or(0);

            let priority = properties
                .manufacturer_data
                .get(&TREADLY_MANUFACTURER_ID)
                .map_or(0, |data| {
                    if data.is_empty() {
                        0
                    } else {
                        i8::try_from(data[0]).unwrap_or(0)
                    }
                });

            let mac_address = Some(properties.address.to_string());

            DeviceInfo {
                name,
                mac_address,
                rssi,
                priority,
                firmware_version: None,
                hardware_version: None,
                serial_number: None,
            }
        } else {
            DeviceInfo::new("Unknown Treadly".to_string(), 0, 0)
        }
    }
}

/// Active connection to a Treadly device
pub struct TreadlyConnection {
    peripheral: Peripheral,
    #[allow(dead_code)]
    tx_char: Characteristic,
    rx_char: Characteristic,
    notification_receiver: mpsc::UnboundedReceiver<Message>,
    _notification_sender: mpsc::UnboundedSender<Message>,
}

impl TreadlyConnection {
    /// Send a command to the device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::Protocol`] if the command fails to send.
    pub async fn send_command(&self, message: &Message) -> Result<()> {
        let data = message.to_bytes();
        debug!("Sending command: {:02X?}", data);

        self.peripheral
            .write(&self.rx_char, &data, WriteType::WithoutResponse)
            .await
            .map_err(|e| TreadlyError::Protocol(format!("Failed to send command: {e}")))?;

        Ok(())
    }

    /// Wait for a notification from the device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::Timeout`] if no notification is received within the timeout,
    /// or [`TreadlyError::Disconnected`] if the device disconnects.
    pub async fn receive_notification(&mut self, timeout_ms: u64) -> Result<Message> {
        timeout(
            Duration::from_millis(timeout_ms),
            self.notification_receiver.recv(),
        )
        .await
        .map_err(|_| TreadlyError::Timeout { timeout_ms })?
        .ok_or_else(|| TreadlyError::Disconnected)
    }

    /// Check if the device is still connected
    pub async fn is_connected(&self) -> bool {
        self.peripheral.is_connected().await.unwrap_or(false)
    }

    /// Disconnect from the device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::Ble`] if disconnection fails.
    pub async fn disconnect(&self) -> Result<()> {
        self.peripheral.disconnect().await?;
        Ok(())
    }

    /// Get device address
    #[must_use]
    pub fn get_address(&self) -> BDAddr {
        self.peripheral.address()
    }
}

/// Handle notifications from the device
///
/// # Errors
///
/// Returns [`TreadlyError::Ble`] if setting up notifications fails.
pub async fn handle_notifications(
    peripheral: Peripheral,
    tx_char: Characteristic,
    sender: mpsc::UnboundedSender<Message>,
) -> Result<()> {
    let mut notification_stream = peripheral.notifications().await?;

    while let Some(data) = notification_stream.next().await {
        if data.uuid == tx_char.uuid {
            if let Ok(message) = Message::from_bytes(&data.value) {
                if sender.send(message).is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ble_manager_creation() {
        let manager = BleManager::new().await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_uuid_parsing() {
        let service_uuid = Uuid::parse_str(TREADLY_SERVICE_UUID);
        assert!(service_uuid.is_ok());

        let tx_uuid = Uuid::parse_str(TREADLY_TX_CHAR_UUID);
        assert!(tx_uuid.is_ok());

        let rx_uuid = Uuid::parse_str(TREADLY_RX_CHAR_UUID);
        assert!(rx_uuid.is_ok());
    }
}
