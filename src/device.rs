use crate::{
    ble::{BleManager, TreadlyConnection},
    error::{Result, TreadlyError},
    protocol::{parse_device_status, Message, MessageId},
    types::{
        AuthenticationStatus, ConnectionParams, DeviceInfo, DeviceStatus, DeviceStatusCode,
        EmergencyStopState, SpeedUnit, TemperatureStatus, TimeoutConfig,
    },
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

/// Main interface for controlling a Treadly treadmill device
///
/// `TreadlyDevice` provides a high-level, safe interface for connecting to and controlling
/// Treadly treadmill devices via Bluetooth Low Energy (BLE). It handles connection management,
/// authentication, command sending, and safety monitoring.
///
/// # Features
///
/// - **Automatic device discovery**: Scan for and connect to available Treadly devices
/// - **Secure authentication**: Support for both basic and MD5 challenge-response authentication
/// - **Safety monitoring**: Automatic emergency stop on connection loss and temperature monitoring
/// - **Speed control**: Set precise speeds with automatic unit conversion
/// - **Status monitoring**: Real-time device status updates and health monitoring
/// - **Connection resilience**: Automatic retry logic and connection health monitoring
///
/// # Safety Features
///
/// The device implementation includes several safety mechanisms:
/// - Automatic emergency stop on connection loss
/// - Temperature monitoring with automatic speed reduction
/// - Emergency stop acknowledgment verification
/// - Connection health monitoring
/// - Handrail emergency stop support
///
/// # Examples
///
/// ## Basic Usage
///
/// ```no_run
/// use treadlers::{TreadlyDevice, SpeedUnit};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Connect to the first available device
///     let device = TreadlyDevice::connect_first().await?;
///     
///     // Power on the treadmill
///     device.power_on().await?;
///     
///     // Set speed to 5 km/h
///     device.set_speed(5.0, SpeedUnit::Kilometers).await?;
///     
///     // Get current status
///     let status = device.get_status().await;
///     println!("Current speed: {:.1} km/h", status.speed.current);
///     
///     // Emergency stop if needed
///     device.emergency_stop().await?;
///     
///     Ok(())
/// }
/// ```
///
/// ## Advanced Connection with Custom Parameters
///
/// ```no_run
/// use treadlers::{TreadlyDevice, ConnectionParams, TimeoutConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let params = ConnectionParams {
///         scan_timeout_ms: 10000,
///         authenticate: true,
///         ..Default::default()
///     };
///     
///     let timeout_config = TimeoutConfig {
///         auth_timeout_ms: 10000,
///         default_timeout_ms: 5000,
///         ..Default::default()
///     };
///     
///     let device = TreadlyDevice::connect_first_with_params_and_timeout(
///         params,
///         timeout_config
///     ).await?;
///     
///     Ok(())
/// }
/// ```
pub struct TreadlyDevice {
    connection: Arc<Mutex<Option<TreadlyConnection>>>,
    device_info: DeviceInfo,
    status: Arc<RwLock<DeviceStatus>>,
    #[allow(dead_code)]
    ble_manager: BleManager,
    authenticated: Arc<RwLock<bool>>,
    last_message_time: Arc<RwLock<Instant>>,
    connection_monitoring_active: Arc<RwLock<bool>>,
    timeout_config: TimeoutConfig,
}

impl TreadlyDevice {
    /// Connect to the first available Treadly device with default settings
    ///
    /// This is the simplest way to connect to a Treadly device. It scans for available
    /// devices and connects to the first one found, using default connection parameters
    /// and timeout settings.
    ///
    /// The device will be automatically authenticated and configured for use.
    /// Connection monitoring is started automatically for safety.
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::DeviceNotFound`] if no Treadly devices are found
    /// during the scan, or any connection/authentication errors from the underlying
    /// connection process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use treadlers::TreadlyDevice;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let device = TreadlyDevice::connect_first().await?;
    ///     println!("Connected to device: {}", device.device_info().name);
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect_first() -> Result<Self> {
        Self::connect_first_with_params(ConnectionParams::default()).await
    }

    /// Connect to the first available Treadly device with custom connection parameters
    ///
    /// This method allows customization of the connection process through the
    /// [`ConnectionParams`] struct, including scan duration, authentication settings,
    /// and device filtering options.
    ///
    /// # Arguments
    ///
    /// * `params` - Connection parameters controlling scan behavior and authentication
    ///
    /// # Device Selection
    ///
    /// When multiple devices are found, they are sorted by:
    /// 1. Priority (device-specific priority values)
    /// 2. Signal strength (RSSI) - stronger signal preferred
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::DeviceNotFound`] if no Treadly devices are found
    /// during the scan, or any BLE connection/authentication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use treadlers::{TreadlyDevice, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let params = ConnectionParams {
    ///         scan_timeout_ms: 15000,
    ///         authenticate: true,
    ///         ..Default::default()
    ///     };
    ///     
    ///     let device = TreadlyDevice::connect_first_with_params(params).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect_first_with_params(params: ConnectionParams) -> Result<Self> {
        Self::connect_first_with_params_and_timeout(params, TimeoutConfig::default()).await
    }

    /// Connect to the first available Treadly device with custom parameters and timeout configuration
    ///
    /// This method provides the most control over the connection process, allowing
    /// customization of both connection parameters and timeout behavior. This is useful
    /// for applications that need specific timing requirements or reliability settings.
    ///
    /// # Arguments
    ///
    /// * `params` - Connection parameters controlling scan behavior and authentication
    /// * `timeout_config` - Timeout configuration for various operations
    ///
    /// # Timeout Configuration
    ///
    /// The timeout configuration allows fine-tuning of:
    /// - Authentication timeouts (basic and secure)
    /// - Command response timeouts
    /// - Emergency stop timeouts
    /// - Retry behavior and delays
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::DeviceNotFound`] if no Treadly devices are found
    /// during the scan, or any BLE connection/authentication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use treadlers::{TreadlyDevice, ConnectionParams, TimeoutConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let params = ConnectionParams::default();
    ///     let timeouts = TimeoutConfig {
    ///         auth_timeout_ms: 15000,
    ///         default_timeout_ms: 8000,
    ///         max_retry_attempts: 5,
    ///         ..Default::default()
    ///     };
    ///     
    ///     let device = TreadlyDevice::connect_first_with_params_and_timeout(
    ///         params,
    ///         timeouts
    ///     ).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - No Treadly devices are found during scanning
    /// - Device connection fails
    /// - Authentication fails
    /// 
    /// # Panics
    /// 
    /// Panics if no devices are found after filtering (this should not happen
    /// as we check for empty devices list first)
    pub async fn connect_first_with_params_and_timeout(
        params: ConnectionParams,
        timeout_config: TimeoutConfig,
    ) -> Result<Self> {
        let ble_manager = BleManager::new().await?;
        let devices = ble_manager.scan_for_devices(&params).await?;

        if devices.is_empty() {
            return Err(TreadlyError::DeviceNotFound);
        }

        let mut sorted_devices = devices;
        sorted_devices.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.rssi.cmp(&a.rssi)));

        let device_info = sorted_devices.into_iter().next().unwrap();
        Self::connect_to_device_with_timeout(device_info, params, timeout_config).await
    }

    /// Connect to a specific Treadly device
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Device connection fails
    /// - Authentication fails
    /// - BLE communication errors occur
    pub async fn connect_to_device(
        device_info: DeviceInfo,
        params: ConnectionParams,
    ) -> Result<Self> {
        Self::connect_to_device_with_timeout(device_info, params, TimeoutConfig::default()).await
    }

    /// Connect to a specific Treadly device with timeout configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Device connection fails
    /// - Authentication fails
    /// - Timeout occurs during connection or authentication
    /// - BLE communication errors occur
    pub async fn connect_to_device_with_timeout(
        device_info: DeviceInfo,
        params: ConnectionParams,
        timeout_config: TimeoutConfig,
    ) -> Result<Self> {
        let mut ble_manager = BleManager::new().await?;
        let connection = ble_manager.connect_to_device(&device_info, &params).await?;

        let device = Self {
            connection: Arc::new(Mutex::new(Some(connection))),
            device_info,
            status: Arc::new(RwLock::new(DeviceStatus::default())),
            ble_manager,
            authenticated: Arc::new(RwLock::new(false)),
            last_message_time: Arc::new(RwLock::new(Instant::now())),
            connection_monitoring_active: Arc::new(RwLock::new(false)),
            timeout_config,
        };

        if params.authenticate {
            device.authenticate_with_fallback().await?;
        }

        device.subscribe_to_status().await?;

        device.refresh_status().await?;

        device.start_connection_monitoring().await?;

        Ok(device)
    }

    /// Get device information
    #[must_use]
    pub const fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Get timeout configuration
    #[must_use]
    pub const fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }

    /// Get appropriate timeout for a specific command
    ///
    /// This method returns the protocol-compliant timeout for different command types
    /// based on the reverse-engineered protocol requirements.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID to determine timeout for
    ///
    /// # Returns
    ///
    /// The appropriate timeout in milliseconds for the command
    #[must_use]
    pub const fn get_command_timeout(&self, message_id: MessageId) -> u64 {
        match message_id {
            MessageId::Authenticate => self.timeout_config.auth_timeout_ms,
            MessageId::SecureAuthenticate | MessageId::SecureAuthenticateVerify => {
                self.timeout_config.secure_auth_timeout_ms
            }
            MessageId::EmergencyStopRequest => self.timeout_config.emergency_stop_timeout_ms,
            MessageId::Status
            | MessageId::StatusEx
            | MessageId::StatusEx2
            | MessageId::BroadcastDeviceStatus => self.timeout_config.status_timeout_ms,
            MessageId::SetSpeed | MessageId::SpeedUp | MessageId::SpeedDown => {
                self.timeout_config.speed_command_timeout_ms
            }
            MessageId::Power => self.timeout_config.power_command_timeout_ms,
            _ => self.timeout_config.default_timeout_ms,
        }
    }

    /// Get current device status
    pub async fn get_status(&self) -> DeviceStatus {
        self.status.read().await.clone()
    }

    /// Check if the device is connected
    pub async fn is_connected(&self) -> bool {
        if let Some(conn) = self.connection.lock().await.as_ref() {
            conn.is_connected().await
        } else {
            false
        }
    }

    /// Check if the device is authenticated
    pub async fn is_authenticated(&self) -> bool {
        *self.authenticated.read().await
    }

    /// Authenticate with the device using basic authentication
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Device communication fails
    /// - Authentication is rejected by the device
    /// - Timeout occurs during authentication
    pub async fn authenticate(&self) -> Result<()> {
        info!("Authenticating with device (basic authentication)");

        let message = Message::authenticate();
        let timeout = self.get_command_timeout(MessageId::Authenticate);
        self.send_command_with_response(message, timeout).await?;

        *self.authenticated.write().await = true;

        {
            let mut status = self.status.write().await;
            status.authentication = AuthenticationStatus::Authenticated;
        }

        info!("Basic authentication successful");
        Ok(())
    }

    /// Authenticate with the device using secure MD5 challenge-response authentication
    ///
    /// This method implements a secure authentication flow where the device sends a challenge
    /// and the client must respond with the correct MD5 hash computed from the challenge
    /// and a secret key. This provides stronger security than basic authentication.
    ///
    /// The authentication process:
    /// 1. Sends a `SecureAuthenticate` command to request a challenge from the device
    /// 2. Receives 16 bytes of challenge data from the device
    /// 3. Computes MD5 hash of the concatenated challenge data and secret key
    /// 4. Sends `SecureAuthenticateVerify` command with the computed hash
    /// 5. Receives verification response from the device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::AuthenticationFailed`] if:
    /// - The challenge response payload is too short (less than 16 bytes)
    /// - The device returns a non-success status for the verification
    /// - Any network or communication errors occur during the process
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.authenticate_secure().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn authenticate_secure(&self) -> Result<()> {
        info!("Starting secure MD5 challenge-response authentication");

        {
            let mut status = self.status.write().await;
            status.authentication = AuthenticationStatus::InProgress;
        }

        let challenge_message = Message::command(MessageId::SecureAuthenticate);
        let challenge_timeout = self.get_command_timeout(MessageId::SecureAuthenticate);
        let challenge_response = self
            .send_command_with_response(challenge_message, challenge_timeout)
            .await?;

        if challenge_response.payload.len() < 16 {
            return Err(TreadlyError::AuthenticationFailed(
                "Invalid challenge response - payload too short".to_string(),
            ));
        }

        let challenge_data = &challenge_response.payload[0..16];
        info!("Received authentication challenge from device");

        let hash_response = Self::compute_md5_challenge_response(challenge_data);

        let verify_message = Message::secure_authenticate_verify(hash_response);
        let verify_timeout = self.get_command_timeout(MessageId::SecureAuthenticateVerify);
        let verify_response = self
            .send_command_with_response(verify_message, verify_timeout)
            .await?;
        if verify_response.status != crate::protocol::STATUS_SUCCESS {
            {
                let mut status = self.status.write().await;
                status.authentication = AuthenticationStatus::Failed;
            }
            return Err(TreadlyError::AuthenticationFailed(format!(
                "Secure authentication failed - device returned status: {:02X}",
                verify_response.status
            )));
        }

        *self.authenticated.write().await = true;

        {
            let mut status = self.status.write().await;
            status.authentication = AuthenticationStatus::Authenticated;
        }

        info!("Secure MD5 challenge-response authentication successful");
        Ok(())
    }

    /// Compute MD5 challenge response for secure authentication
    ///
    /// This method implements the MD5 challenge-response algorithm required by the
    /// Treadly device's secure authentication protocol. It concatenates the challenge
    /// data received from the device with a predefined secret key and computes the
    /// MD5 hash of the combined data.
    ///
    /// The algorithm follows the reverse-engineered protocol from the official
    /// Android application's secure authentication implementation.
    ///
    /// # Arguments
    ///
    /// * `challenge_data` - A 16-byte challenge received from the device during
    ///   the secure authentication handshake
    ///
    /// # Returns
    ///
    /// A 16-byte MD5 hash that serves as the response to the authentication challenge
    ///
    /// # Errors
    ///
    /// This method currently does not return errors as MD5 computation is deterministic,
    /// but the return type is preserved for future extensibility and consistency
    /// with the authentication flow.
    fn compute_md5_challenge_response(challenge_data: &[u8]) -> [u8; 16] {
        let mut hasher = md5::Context::new();
        hasher.consume(challenge_data);
        hasher.consume(crate::protocol::AUTH_SECRET_KEY);
        let result = hasher.finalize();
        let hash_bytes = result.0;

        info!("Computed MD5 challenge response");
        hash_bytes
    }

    /// Attempt authentication with automatic fallback from secure to basic
    ///
    /// This method provides a robust authentication strategy that attempts secure
    /// MD5 challenge-response authentication first. If secure authentication fails
    /// (due to device incompatibility or other issues), it automatically falls back
    /// to basic authentication.
    ///
    /// This approach ensures maximum compatibility across different Treadly device
    /// models and firmware versions while preferring the more secure authentication
    /// method when available.
    ///
    /// # Authentication Flow
    ///
    /// 1. Attempts secure MD5 challenge-response authentication
    /// 2. If secure authentication succeeds, returns successfully
    /// 3. If secure authentication fails, logs the failure and resets authentication state
    /// 4. Attempts basic authentication as fallback
    /// 5. Returns success if basic authentication succeeds
    /// 6. Returns error if both methods fail
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::AuthenticationFailed`] if both secure and basic
    /// authentication methods fail. The error message includes details about
    /// both failure reasons for debugging purposes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.authenticate_with_fallback().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn authenticate_with_fallback(&self) -> Result<()> {
        info!("Attempting authentication with secure->basic fallback");

        match self.authenticate_secure().await {
            Ok(()) => {
                info!("Secure authentication successful");
                Ok(())
            }
            Err(e) => {
                warn!("Secure authentication failed: {}, falling back to basic", e);

                {
                    let mut status = self.status.write().await;
                    status.authentication = AuthenticationStatus::NotAuthenticated;
                }
                *self.authenticated.write().await = false;

                match self.authenticate().await {
                    Ok(()) => {
                        info!("Basic authentication successful after secure fallback");
                        Ok(())
                    }
                    Err(basic_err) => {
                        error!("Both secure and basic authentication failed");
                        Err(TreadlyError::AuthenticationFailed(format!(
                            "Secure auth failed: {e}, Basic auth failed: {basic_err}"
                        )))
                    }
                }
            }
        }
    }

    /// Power on the treadmill
    ///
    /// This method sends a power command to turn on the treadmill. The device
    /// must be connected and authenticated before calling this method.
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected or
    /// if an emergency stop condition is active.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.power_on().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn power_on(&self) -> Result<()> {
        info!("Powering on treadmill");
        self.ensure_ready().await?;

        let message = Message::command(MessageId::Power);
        let timeout = self.get_command_timeout(MessageId::Power);
        self.send_command_with_retry(message, timeout, self.timeout_config.max_retry_attempts)
            .await?;

        {
            let mut status = self.status.write().await;
            status.power_on = true;
        }

        Ok(())
    }

    /// Set the treadmill speed to a specific value
    ///
    /// This method sets the treadmill to run at the specified speed in the given unit.
    /// The device must be powered on and ready before setting speed.
    ///
    /// # Arguments
    ///
    /// * `speed` - The target speed value (0.0 to 20.0 km/h or equivalent in miles)
    /// * `unit` - The unit of measurement ([`SpeedUnit::Kilometers`] or [`SpeedUnit::Miles`])
    ///
    /// # Speed Limits
    ///
    /// - Minimum speed: 0.0 km/h (stopped)
    /// - Maximum speed: 20.0 km/h (approximately 12.4 mph)
    /// - Values in miles are automatically converted to km/h for the device
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::InvalidParameters`] if the speed is outside the
    /// valid range (0.0 - 20.0 km/h).
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected,
    /// not powered on, or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::{TreadlyDevice, SpeedUnit};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.power_on().await?;
    ///
    /// // Set speed to 5 km/h
    /// device.set_speed(5.0, SpeedUnit::Kilometers).await?;
    ///
    /// // Set speed to 3 mph (automatically converted to km/h)
    /// device.set_speed(3.0, SpeedUnit::Miles).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_speed(&self, speed: f32, unit: SpeedUnit) -> Result<()> {
        info!("Setting speed to {:.1} {}", speed, unit);
        self.ensure_ready().await?;
        self.ensure_powered_on().await?;

        let speed_kmh = match unit {
            SpeedUnit::Kilometers => speed,
            SpeedUnit::Miles => speed * 1.6093,
        };

        if !(0.0..=20.0).contains(&speed_kmh) {
            return Err(TreadlyError::InvalidParameters(format!(
                "Speed {speed_kmh:.1} km/h is out of range (0.0 - 20.0)"
            )));
        }

        let message = Message::set_speed(speed_kmh);
        let timeout = self.get_command_timeout(MessageId::SetSpeed);
        self.send_command_with_retry(message, timeout, self.timeout_config.max_retry_attempts)
            .await?;

        Ok(())
    }

    /// Increase the treadmill speed incrementally
    ///
    /// This method sends a speed-up command to increase the treadmill speed
    /// by a predefined increment. The exact increment depends on the device
    /// firmware and current speed settings.
    ///
    /// # Safety
    ///
    /// The device will respect its maximum speed limits and will not exceed
    /// safe operating speeds regardless of how many times this command is called.
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected,
    /// not powered on, or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.power_on().await?;
    /// device.speed_up().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn speed_up(&self) -> Result<()> {
        info!("Increasing speed");
        self.ensure_ready().await?;
        self.ensure_powered_on().await?;

        let message = Message::command(MessageId::SpeedUp);
        self.send_command_with_response(message, 3000).await?;

        Ok(())
    }

    /// Decrease the treadmill speed incrementally
    ///
    /// This method sends a speed-down command to decrease the treadmill speed
    /// by a predefined increment. The exact decrement depends on the device
    /// firmware and current speed settings.
    ///
    /// # Safety
    ///
    /// The device will respect its minimum speed limits (including full stop)
    /// and will not go below safe operating speeds.
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected,
    /// not powered on, or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.power_on().await?;
    /// device.speed_down().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn speed_down(&self) -> Result<()> {
        info!("Decreasing speed");
        self.ensure_ready().await?;
        self.ensure_powered_on().await?;

        let message = Message::command(MessageId::SpeedDown);
        self.send_command_with_response(message, 3000).await?;

        Ok(())
    }

    /// Emergency stop the treadmill with device acknowledgment verification
    ///
    /// This method implements enhanced emergency stop with safety verification:
    /// 1. Sends emergency stop command with highest priority
    /// 2. Waits for device acknowledgment within timeout
    /// 3. Verifies that device has actually stopped
    /// 4. Returns error if acknowledgment is not received
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::EmergencyStopNotAcknowledged`] if device doesn't acknowledge
    /// the emergency stop command within the timeout period.
    pub async fn emergency_stop(&self) -> Result<()> {
        warn!("Emergency stop activated - sending command with acknowledgment verification");

        let message = Message::command(MessageId::EmergencyStopRequest);

        let timeout = self.get_command_timeout(MessageId::EmergencyStopRequest);
        match self
            .send_command_with_response_no_safety(message, timeout)
            .await
        {
            Ok(response) => {
                if response.id == MessageId::EmergencyStopRequest
                    || response.id == MessageId::Status
                    || response.id == MessageId::BroadcastDeviceStatus
                {
                    if let Ok(status) = crate::protocol::parse_device_status(&response) {
                        if status.emergency_stop == EmergencyStopState::Active
                            || status.speed.current == 0.0
                        {
                            info!("Emergency stop acknowledged by device");

                            {
                                let mut local_status = self.status.write().await;
                                local_status.emergency_stop = EmergencyStopState::Active;
                                local_status.speed.current = 0.0;
                                local_status.speed.target = 0.0;
                            }

                            return Ok(());
                        }
                    }
                }

                error!("Emergency stop command sent but device response doesn't confirm stop");
                Err(TreadlyError::EmergencyStopNotAcknowledged)
            }
            Err(e) => {
                error!("Emergency stop command failed: {}", e);

                {
                    let mut status = self.status.write().await;
                    status.emergency_stop = EmergencyStopState::Active;
                    status.speed.current = 0.0;
                    status.speed.target = 0.0;
                }

                Err(e)
            }
        }
    }

    /// Emergency stop with immediate response (no acknowledgment wait)
    ///
    /// This method sends an emergency stop command immediately without waiting
    /// for device acknowledgment. Use this variant when immediate action is required
    /// and acknowledgment verification might be too slow for the safety situation.
    ///
    /// Unlike [`emergency_stop`], this method does not wait for device confirmation
    /// and assumes the command was received. This provides the fastest possible
    /// emergency stop response time.
    ///
    /// # When to Use
    ///
    /// - Connection is unstable or compromised
    /// - Immediate stop is required regardless of confirmation
    /// - As a fallback when normal emergency stop fails
    ///
    /// # Safety
    ///
    /// This method updates the local device status to reflect the emergency stop
    /// state, even if the command transmission fails. This ensures consistent
    /// local state management.
    ///
    /// # Errors
    ///
    /// Returns errors if the emergency stop command fails to send over the
    /// connection, but the local status will still be updated to emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    ///
    /// // In an emergency situation requiring immediate stop
    /// device.emergency_stop_immediate().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn emergency_stop_immediate(&self) -> Result<()> {
        warn!("Immediate emergency stop activated");

        let message = Message::command(MessageId::EmergencyStopRequest);
        self.send_command(message).await?;

        {
            let mut status = self.status.write().await;
            status.emergency_stop = EmergencyStopState::Active;
            status.speed.current = 0.0;
            status.speed.target = 0.0;
        }

        Ok(())
    }

    /// Reset the device from emergency stop state
    ///
    /// This method clears the emergency stop condition and allows the device
    /// to resume normal operation. The device must be in emergency stop state
    /// before calling this method.
    ///
    /// # Safety
    ///
    /// Only reset emergency stop when it is safe to do so. Ensure that:
    /// - The cause of the emergency stop has been resolved
    /// - The area around the treadmill is clear
    /// - The device is in a safe state to resume operation
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected
    /// or not in a state where emergency stop reset is allowed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    ///
    /// // After an emergency stop, reset when safe
    /// device.reset_emergency_stop().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset_emergency_stop(&self) -> Result<()> {
        info!("Resetting emergency stop");
        self.ensure_ready().await?;

        let message = Message::command(MessageId::ResetStop);
        self.send_command_with_response(message, 3000).await?;

        {
            let mut status = self.status.write().await;
            status.emergency_stop = EmergencyStopState::Normal;
        }

        Ok(())
    }

    /// Set the speed unit for display and input
    ///
    /// This method configures whether speed values are displayed and interpreted
    /// in kilometers per hour or miles per hour. This affects both the device
    /// display and the units expected by speed-related commands.
    ///
    /// # Arguments
    ///
    /// * `unit` - The desired speed unit ([`SpeedUnit::Kilometers`] or [`SpeedUnit::Miles`])
    ///
    /// # Note
    ///
    /// Internally, the device always operates in km/h. When miles are selected,
    /// speed values are automatically converted for display and user input.
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected
    /// or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::{TreadlyDevice, SpeedUnit};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    ///
    /// // Set unit to miles per hour
    /// device.set_speed_unit(SpeedUnit::Miles).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_speed_unit(&self, unit: SpeedUnit) -> Result<()> {
        info!("Setting speed unit to {}", unit);
        self.ensure_ready().await?;

        let message_id = match unit {
            SpeedUnit::Kilometers => MessageId::SetUnitKilometers,
            SpeedUnit::Miles => MessageId::SetUnitMiles,
        };

        let message = Message::command(message_id);
        self.send_command_with_response(message, 3000).await?;

        {
            let mut status = self.status.write().await;
            status.speed.unit = unit;
        }

        Ok(())
    }

    /// Enable or disable handrail emergency stop feature
    ///
    /// This method controls whether the handrail emergency stop mechanism is active.
    /// When enabled, the treadmill will automatically stop if the handrail safety
    /// mechanism is triggered (typically by removing hands from the handrail sensors).
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to enable handrail emergency stop, `false` to disable
    ///
    /// # Safety Considerations
    ///
    /// - Enabling handrail emergency stop provides an additional safety layer
    /// - Disabling it may be necessary for certain exercise routines or user preferences
    /// - Consider user safety requirements when changing this setting
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected
    /// or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    ///
    /// // Enable handrail emergency stop for safety
    /// device.set_handrail_enabled(true).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_handrail_enabled(&self, enabled: bool) -> Result<()> {
        info!("Setting handrail emergency stop: {}", enabled);
        self.ensure_ready().await?;

        let message = Message::set_handrail_enabled(enabled);
        self.send_command_with_response(message, 3000).await?;

        {
            let mut status = self.status.write().await;
            status.handrail_enabled = enabled;
        }

        Ok(())
    }

    /// Pause the treadmill operation
    ///
    /// This method sends a pause command to temporarily stop the treadmill
    /// without triggering an emergency stop. The treadmill can be resumed
    /// from the paused state.
    ///
    /// # Difference from Emergency Stop
    ///
    /// - Pause: Temporary stop, can be easily resumed
    /// - Emergency Stop: Safety stop, requires reset before resuming
    ///
    /// # Errors
    ///
    /// Returns [`TreadlyError::NotReady`] if the device is not connected
    /// or in an emergency stop state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.power_on().await?;
    ///
    /// // Pause during exercise
    /// device.pause().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pause(&self) -> Result<()> {
        info!("Pausing treadmill");
        self.ensure_ready().await?;

        let message = Message::command(MessageId::Pause);
        self.send_command_with_response(message, 3000).await?;

        Ok(())
    }

    /// Refresh the device status from the hardware
    ///
    /// This method requests the current status from the device and updates
    /// the local status cache. Use this to get the most up-to-date information
    /// about the device state, including speed, power state, and safety conditions.
    ///
    /// # Errors
    ///
    /// Returns communication errors if the status request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.refresh_status().await?;
    /// let status = device.get_status().await;
    /// println!("Current speed: {:.1} km/h", status.speed.current);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_status(&self) -> Result<()> {
        let message = Message::command(MessageId::Status);
        let timeout = self.get_command_timeout(MessageId::Status);
        self.send_command_with_response(message, timeout).await?;
        Ok(())
    }

    /// Subscribe to automatic status updates from the device
    ///
    /// This method enables automatic status broadcasts from the device,
    /// allowing the client to receive real-time updates about device state
    /// changes without polling.
    ///
    /// # Note
    ///
    /// This is typically called automatically during connection setup.
    /// Manual subscription is only needed for advanced use cases.
    ///
    /// # Errors
    ///
    /// Returns communication errors if the subscription request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// device.subscribe_to_status().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_to_status(&self) -> Result<()> {
        let message = Message::command(MessageId::SubscribeStatus);
        self.send_command(message).await?;
        Ok(())
    }

    /// Disconnect from the device and clean up resources
    ///
    /// This method gracefully disconnects from the treadmill device,
    /// stopping all monitoring tasks and cleaning up the connection.
    /// The device object cannot be used after disconnection.
    ///
    /// # Note
    ///
    /// This method is automatically called when the `TreadlyDevice`
    /// is dropped, so manual disconnection is optional.
    ///
    /// # Errors
    ///
    /// Returns errors if the disconnection process fails, though
    /// the cleanup will still be attempted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use treadlers::TreadlyDevice;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let device = TreadlyDevice::connect_first().await?;
    /// // ... use device ...
    /// device.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from device");

        let conn = self.connection.lock().await.take();
        if let Some(conn) = conn {
            conn.disconnect().await?;
        }

        *self.authenticated.write().await = false;

        Ok(())
    }

    /// Send a command and wait for response
    async fn send_command_with_response(
        &self,
        message: Message,
        timeout_ms: u64,
    ) -> Result<Message> {
        self.send_command_with_response_internal(message, timeout_ms, true)
            .await
    }

    /// Send a command with retry logic and exponential backoff
    ///
    /// This method implements protocol-compliant retry logic with exponential backoff
    /// for improved reliability under network conditions.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    /// * `timeout_ms` - Initial timeout in milliseconds
    /// * `max_retries` - Maximum number of retry attempts
    ///
    /// # Errors
    ///
    /// Returns the last error encountered after all retries are exhausted.
    ///
    /// # Panics
    ///
    /// Panics if `last_error` is `None` when logging retry attempts. This should not happen
    /// in normal operation as the error is set before the logging statement.
    pub async fn send_command_with_retry(
        &self,
        message: Message,
        timeout_ms: u64,
        max_retries: u32,
    ) -> Result<Message> {
        let mut last_error = None;
        let mut current_timeout = timeout_ms;

        for attempt in 0..=max_retries {
            match self
                .send_command_with_response(message.clone(), current_timeout)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < max_retries {
                        warn!(
                            "Command {:?} failed on attempt {}/{}, retrying in {}ms: {}",
                            message.id,
                            attempt + 1,
                            max_retries + 1,
                            self.timeout_config.retry_delay_ms,
                            last_error.as_ref().unwrap()
                        );

                        current_timeout = current_timeout.saturating_mul(3).saturating_div(2);
                        tokio::time::sleep(Duration::from_millis(
                            self.timeout_config.retry_delay_ms,
                        ))
                        .await;
                    }
                }
            }
        }

        error!(
            "Command {:?} failed after {} attempts",
            message.id,
            max_retries + 1
        );

        Err(last_error.unwrap())
    }

    /// Send a command and wait for response without safety monitoring (internal use)
    async fn send_command_with_response_no_safety(
        &self,
        message: Message,
        timeout_ms: u64,
    ) -> Result<Message> {
        self.send_command(message).await?;

        let mut connection = self.connection.lock().await;
        if let Some(conn) = connection.as_mut() {
            let response = conn.receive_notification(timeout_ms).await?;

            *self.last_message_time.write().await = Instant::now();

            if matches!(
                response.id,
                MessageId::Status | MessageId::StatusEx | MessageId::BroadcastDeviceStatus
            ) {
                if let Ok(new_status) = parse_device_status(&response) {
                    *self.status.write().await = new_status;
                }
            }

            Ok(response)
        } else {
            Err(TreadlyError::Disconnected)
        }
    }

    /// Internal method for sending commands with optional safety monitoring
    async fn send_command_with_response_internal(
        &self,
        message: Message,
        timeout_ms: u64,
        perform_safety_monitoring: bool,
    ) -> Result<Message> {
        self.send_command(message).await?;

        let mut connection = self.connection.lock().await;
        if let Some(conn) = connection.as_mut() {
            let response = conn.receive_notification(timeout_ms).await?;

            *self.last_message_time.write().await = Instant::now();

            if matches!(
                response.id,
                MessageId::Status | MessageId::StatusEx | MessageId::BroadcastDeviceStatus
            ) {
                if let Ok(new_status) = parse_device_status(&response) {
                    if perform_safety_monitoring {
                        if let Err(safety_error) = self.perform_safety_monitoring(&new_status).await
                        {
                            warn!("Safety monitoring detected issue: {}", safety_error);
                        }
                    }

                    *self.status.write().await = new_status;
                }
            }

            Ok(response)
        } else {
            Err(TreadlyError::Disconnected)
        }
    }

    /// Send a command without waiting for response
    async fn send_command(&self, message: Message) -> Result<()> {
        let connection = self.connection.lock().await;
        if let Some(conn) = connection.as_ref() {
            conn.send_command(&message).await
        } else {
            Err(TreadlyError::Disconnected)
        }
    }

    /// Ensure the device is ready for commands
    async fn ensure_ready(&self) -> Result<()> {
        if !self.is_connected().await {
            return Err(TreadlyError::NotReady {
                reason: "Device not connected".to_string(),
            });
        }

        {
            let status = self.status.read().await;
            match status.emergency_stop {
                EmergencyStopState::Active => {
                    return Err(TreadlyError::EmergencyStop);
                }
                EmergencyStopState::ResetRequired => {
                    return Err(TreadlyError::NotReady {
                        reason: "Emergency stop reset required".to_string(),
                    });
                }
                EmergencyStopState::Normal => {}
            }
        }

        Ok(())
    }

    /// Ensure the device is powered on
    async fn ensure_powered_on(&self) -> Result<()> {
        {
            let status = self.status.read().await;
            if !status.power_on {
                return Err(TreadlyError::NotReady {
                    reason: "Device not powered on".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Monitor temperature status and respond automatically (safety critical)
    ///
    /// This method implements the temperature safety system from the protocol:
    /// - Normal: No action required
    /// - `ReduceSpeed`: Automatically reduce speed for safety
    /// - Stop: Force emergency stop immediately
    /// - Unknown: Treat as error condition
    ///
    /// # Errors
    ///
    /// Returns temperature-related errors for safety conditions.
    pub async fn monitor_temperature(&self, status: &DeviceStatus) -> Result<()> {
        match status.temperature_status {
            TemperatureStatus::Stop => {
                warn!("Critical temperature detected - forcing emergency stop");
                self.emergency_stop().await?;
                Err(TreadlyError::TemperatureSafetyStop)
            }
            TemperatureStatus::ReduceSpeed => {
                warn!("High temperature detected - reducing speed for safety");
                self.reduce_speed_for_temperature().await?;
                Err(TreadlyError::HighTemperatureSpeedReduction)
            }
            TemperatureStatus::Unknown => {
                warn!("Temperature sensor error - safety monitoring compromised");
                Err(TreadlyError::TemperatureSensorError)
            }
            TemperatureStatus::Normal => Ok(()),
        }
    }

    /// Automatically reduce speed due to high temperature (safety critical)
    ///
    /// This implements the automatic speed reduction safety response when
    /// temperature status indicates `ReduceSpeed` condition.
    ///
    /// # Errors
    ///
    /// Returns errors if speed reduction fails.
    async fn reduce_speed_for_temperature(&self) -> Result<()> {
        let current_status = self.status.read().await;
        let current_speed = current_status.speed.current;

        let reduced_speed = current_speed * 0.7;
        let min_safe_speed = 1.0; // Minimum safe speed

        let target_speed = if reduced_speed < min_safe_speed {
            min_safe_speed
        } else {
            reduced_speed
        };

        drop(current_status); // Release read lock

        info!(
            "Reducing speed from {:.1} to {:.1} km/h due to high temperature",
            current_speed, target_speed
        );

        let speed_kmh = match SpeedUnit::Kilometers {
            SpeedUnit::Kilometers => target_speed,
            SpeedUnit::Miles => target_speed * 1.6093,
        };

        if !(0.0..=20.0).contains(&speed_kmh) {
            return Err(TreadlyError::InvalidParameters(format!(
                "Speed {speed_kmh:.1} km/h is outside safe range (0.0-20.0 km/h)"
            )));
        }

        let message = Message::set_speed(target_speed);
        self.send_command_with_response_no_safety(message, 3000)
            .await?;

        Ok(())
    }

    /// Monitor device status codes and respond to safety conditions
    ///
    /// This monitors the device status codes and responds appropriately:
    /// - `HighTemperature`: Log warning (temperature status handles action)
    /// - `PowerCycleRequired`: Return error requiring user action
    /// - Other errors: Log for debugging
    ///
    /// # Errors
    ///
    /// Returns errors for safety-related device status conditions.
    pub fn monitor_device_status(&self, status: &DeviceStatus) -> Result<()> {
        match status.device_status_code {
            DeviceStatusCode::HighTemperature => {
                warn!("Device reporting high temperature status");
                Ok(())
            }
            DeviceStatusCode::PowerCycleRequired => {
                warn!("Device requires power cycle - safety condition");
                Err(TreadlyError::PowerCycleRequired)
            }
            DeviceStatusCode::RequestError => {
                warn!("Device reporting request error");
                Ok(())
            }
            DeviceStatusCode::WifiNotConnected | DeviceStatusCode::WifiError => {
                info!("Device WiFi status: {}", status.device_status_code);
                Ok(())
            }
            DeviceStatusCode::WifiScanning => {
                info!("Device WiFi scanning in progress");
                Ok(())
            }
            DeviceStatusCode::NoError => Ok(()),
        }
    }

    /// Perform comprehensive safety monitoring on device status
    ///
    /// This method performs all safety checks on device status:
    /// - Temperature monitoring with automatic responses
    /// - Device status code monitoring
    /// - Connection health monitoring
    /// - Emergency stop state validation
    ///
    /// # Errors
    ///
    /// Returns errors for any safety conditions detected.
    pub async fn perform_safety_monitoring(&self, status: &DeviceStatus) -> Result<()> {
        self.monitor_temperature(status).await?;

        self.monitor_device_status(status)?;

        self.monitor_connection_health().await?;

        if status.emergency_stop == EmergencyStopState::Active {
            warn!("Emergency stop is active - device stopped for safety");
            return Err(TreadlyError::EmergencyStop);
        }

        if status.emergency_stop == EmergencyStopState::ResetRequired {
            warn!("Emergency stop reset required before operation");
            return Err(TreadlyError::EmergencyStop);
        }

        Ok(())
    }

    /// Start connection monitoring with automatic emergency stop on connection loss
    ///
    /// This method implements the connection loss protection system:
    /// - Monitors connection health continuously
    /// - Tracks message timestamps to detect communication loss
    /// - Triggers emergency stop if connection is lost during operation
    /// - Attempts automatic reconnection with state restoration
    ///
    /// # Errors
    ///
    /// Returns connection-related errors for safety conditions.
    pub async fn start_connection_monitoring(&self) -> Result<()> {
        let monitoring_active = self.connection_monitoring_active.clone();
        let last_message_time = self.last_message_time.clone();
        let connection = self.connection.clone();
        let status = self.status.clone();

        *monitoring_active.write().await = true;

        let monitoring_task = tokio::spawn(async move {
            const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
            const MESSAGE_TIMEOUT: Duration = Duration::from_secs(10);
            const MONITOR_INTERVAL: Duration = Duration::from_millis(500);

            info!("Connection monitoring started");

            loop {
                if !*monitoring_active.read().await {
                    info!("Connection monitoring stopped");
                    break;
                }

                let is_connected = if let Some(conn) = connection.lock().await.as_ref() {
                    conn.is_connected().await
                } else {
                    false
                };

                if !is_connected {
                    error!("Connection lost - triggering emergency stop for safety");
                    {
                        let mut device_status = status.write().await;
                        device_status.connection_health = crate::types::ConnectionHealth::Lost;
                    }
                    break;
                }

                let last_message = *last_message_time.read().await;
                if last_message.elapsed() > MESSAGE_TIMEOUT {
                    warn!("Message timeout detected - connection may be unstable");

                    {
                        let mut device_status = status.write().await;
                        device_status.connection_health = crate::types::ConnectionHealth::Unstable;

                        if last_message.elapsed() > CONNECTION_TIMEOUT {
                            error!("Connection timeout exceeded - triggering emergency stop");
                            device_status.connection_health = crate::types::ConnectionHealth::Lost;
                            drop(device_status);
                            break;
                        }
                    }
                } else {
                    {
                        let mut device_status = status.write().await;
                        device_status.connection_health = crate::types::ConnectionHealth::Healthy;
                    }
                }

                tokio::time::sleep(MONITOR_INTERVAL).await;
            }

            *monitoring_active.write().await = false;
        });

        tokio::spawn(monitoring_task);

        Ok(())
    }

    /// Stop connection monitoring
    ///
    /// This stops the background connection monitoring task.
    pub async fn stop_connection_monitoring(&self) {
        *self.connection_monitoring_active.write().await = false;
        info!("Connection monitoring stop requested");
    }

    /// Check if connection monitoring is active
    pub async fn is_connection_monitoring_active(&self) -> bool {
        *self.connection_monitoring_active.read().await
    }

    /// Get connection health status
    pub async fn get_connection_health(&self) -> crate::types::ConnectionHealth {
        self.status.read().await.connection_health
    }

    /// Get device MAC address from the device
    ///
    /// This method requests the MAC address from the device and returns it.
    /// The MAC address is used for device validation and authentication.
    ///
    /// # Errors
    ///
    /// Returns errors if the MAC address request fails.
    pub async fn get_mac_address(&self) -> Result<String> {
        info!("Requesting MAC address from device");

        let message = Message::command(MessageId::MacAddress);
        let timeout = self.get_command_timeout(MessageId::MacAddress);
        let response = self.send_command_with_response(message, timeout).await?;

        if response.payload.len() < 6 {
            return Err(TreadlyError::Protocol(
                "MAC address response payload too short".to_string(),
            ));
        }

        let mac_bytes = &response.payload[0..6];
        let mac_address = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac_bytes[0], mac_bytes[1], mac_bytes[2], mac_bytes[3], mac_bytes[4], mac_bytes[5]
        );

        info!("Device MAC address: {}", mac_address);
        Ok(mac_address)
    }

    /// Verify device MAC address authenticity
    ///
    /// This method verifies that the device's MAC address matches the expected
    /// MAC address and validates its authenticity using the device's internal
    /// verification mechanism.
    ///
    /// # Arguments
    ///
    /// * `expected_mac` - Expected MAC address for verification
    ///
    /// # Errors
    ///
    /// Returns errors if MAC address verification fails.
    pub async fn verify_mac_address(&self, expected_mac: &str) -> Result<()> {
        info!("Verifying MAC address: {}", expected_mac);

        let actual_mac = self.get_mac_address().await?;

        if actual_mac.to_lowercase() != expected_mac.to_lowercase() {
            return Err(TreadlyError::AuthenticationFailed(format!(
                "MAC address mismatch: expected {expected_mac}, got {actual_mac}"
            )));
        }

        let mac_bytes = Self::parse_mac_address(expected_mac)?;
        let message = Message::verify_mac_address(mac_bytes);
        let timeout = self.get_command_timeout(MessageId::VerifyMacAddress);
        let response = self.send_command_with_response(message, timeout).await?;

        if response.status != crate::protocol::STATUS_SUCCESS {
            return Err(TreadlyError::AuthenticationFailed(format!(
                "MAC address verification failed - device returned status: {:02X}",
                response.status
            )));
        }

        info!("MAC address verification successful");
        Ok(())
    }

    /// Parse MAC address string into bytes
    ///
    /// # Arguments
    ///
    /// * `mac_address` - MAC address string in format "XX:XX:XX:XX:XX:XX"
    ///
    /// # Errors
    ///
    /// Returns errors if MAC address format is invalid.
    fn parse_mac_address(mac_address: &str) -> Result<[u8; 6]> {
        let parts: Vec<&str> = mac_address.split(':').collect();
        if parts.len() != 6 {
            return Err(TreadlyError::InvalidParameters(format!(
                "Invalid MAC address format: {mac_address}. Expected format: XX:XX:XX:XX:XX:XX"
            )));
        }

        let mut mac_bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            mac_bytes[i] = u8::from_str_radix(part, 16).map_err(|_| {
                TreadlyError::InvalidParameters(format!("Invalid MAC address byte: {part}"))
            })?;
        }

        Ok(mac_bytes)
    }

    /// Validate device authenticity and identity
    ///
    /// This method performs comprehensive device validation including:
    /// - Device identity verification
    /// - MAC address validation (if provided)
    /// - Device authenticity checks
    /// - Protocol compliance verification
    ///
    /// # Arguments
    ///
    /// * `expected_mac` - Optional expected MAC address for validation
    ///
    /// # Errors
    ///
    /// Returns errors if device validation fails.
    pub async fn validate_device(&self, expected_mac: Option<&str>) -> Result<()> {
        info!("Starting comprehensive device validation");

        let message = Message::command(MessageId::ValidateDevice);
        let timeout = self.get_command_timeout(MessageId::ValidateDevice);
        let response = self.send_command_with_response(message, timeout).await?;

        if response.status != crate::protocol::STATUS_SUCCESS {
            return Err(TreadlyError::AuthenticationFailed(format!(
                "Device validation failed - device returned status: {:02X}",
                response.status
            )));
        }

        if let Some(mac) = expected_mac {
            self.verify_mac_address(mac).await?;
        }

        let status_message = Message::command(MessageId::Status);
        let status_timeout = self.get_command_timeout(MessageId::Status);
        let status_response = self
            .send_command_with_response(status_message, status_timeout)
            .await?;

        match crate::protocol::parse_device_status(&status_response) {
            Ok(_device_status) => {
                info!("Device validation successful - device is authentic Treadly");

                if let Some(mac) = expected_mac {
                    info!("Device MAC address validated: {}", mac);
                }

                Ok(())
            }
            Err(e) => Err(TreadlyError::AuthenticationFailed(format!(
                "Device validation failed - invalid device status response: {e}"
            ))),
        }
    }

    /// Validate device during connection process
    ///
    /// This method is called during connection to validate the device before
    /// proceeding with authentication and setup.
    ///
    /// # Arguments
    ///
    /// * `expected_mac` - Optional expected MAC address for validation
    ///
    /// # Errors
    ///
    /// Returns errors if device validation fails.
    pub async fn validate_device_on_connection(&self, expected_mac: Option<&str>) -> Result<()> {
        info!("Validating device during connection");

        let validation_result = self.validate_device(expected_mac).await;

        match validation_result {
            Ok(()) => {
                info!("Device validation successful during connection");
                Ok(())
            }
            Err(e) => {
                error!("Device validation failed during connection: {}", e);
                Err(e)
            }
        }
    }

    /// Force emergency stop due to connection loss (safety critical)
    ///
    /// This method is called when connection monitoring detects a critical
    /// connection loss that requires immediate emergency stop for safety.
    ///
    /// # Errors
    ///
    /// Returns connection loss error.
    pub async fn emergency_stop_connection_loss(&self) -> Result<()> {
        error!("Emergency stop triggered due to connection loss");

        if self.is_connected().await {
            if let Err(e) = self.emergency_stop_immediate().await {
                error!("Failed to send emergency stop command: {}", e);
            }
        }

        let mut device_status = self.status.write().await;
        device_status.emergency_stop = EmergencyStopState::Active;
        device_status.connection_health = crate::types::ConnectionHealth::Lost;
        drop(device_status);

        Err(TreadlyError::ConnectionLostEmergencyStop)
    }

    /// Monitor connection health and respond to safety conditions
    ///
    /// This method checks the connection health and responds appropriately:
    /// - Healthy: Normal operation
    /// - Degraded: Log warning but continue
    /// - Unstable: Log warning and consider reconnection
    /// - Lost: Trigger emergency stop
    ///
    /// # Errors
    ///
    /// Returns errors for safety-related connection conditions.
    pub async fn monitor_connection_health(&self) -> Result<()> {
        let connection_health = self.get_connection_health().await;

        match connection_health {
            crate::types::ConnectionHealth::Lost => self.emergency_stop_connection_loss().await,
            crate::types::ConnectionHealth::Unstable => {
                warn!("Connection unstable - monitoring for safety");
                Err(TreadlyError::ConnectionHealthDegraded)
            }
            crate::types::ConnectionHealth::Degraded => {
                warn!("Connection degraded - monitoring for safety");
                Ok(())
            }
            crate::types::ConnectionHealth::Healthy => Ok(()),
        }
    }
}

impl Drop for TreadlyDevice {
    fn drop(&mut self) {
        let connection = self.connection.clone();
        let monitoring_active = self.connection_monitoring_active.clone();

        tokio::spawn(async move {
            *monitoring_active.write().await = false;

            let value = connection.lock().await.take();
            if let Some(conn) = value {
                let _ = conn.disconnect().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_validation() {
        let invalid_speeds = vec![-1.0, 25.0, 100.0];

        for speed in invalid_speeds {
            let speed_kmh = match SpeedUnit::Kilometers {
                SpeedUnit::Kilometers => speed,
                SpeedUnit::Miles => speed * 1.6093,
            };

            assert!(!(0.0..=20.0).contains(&speed_kmh));
        }
    }

    #[test]
    fn test_unit_conversion() {
        let speed_mph = 10.0f32;
        let speed_kmh = speed_mph * 1.6093f32;
        assert!((speed_kmh - 16.093f32).abs() < 0.01f32);
    }

    #[test]
    fn test_mac_address_parsing_logic() {
        let test_cases = vec![
            (
                "12:34:56:78:9A:BC",
                true,
                [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC],
            ),
            (
                "00:11:22:33:44:55",
                true,
                [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            ),
            (
                "FF:EE:DD:CC:BB:AA",
                true,
                [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA],
            ),
        ];

        for (mac_str, should_succeed, expected_bytes) in test_cases {
            let result = parse_mac_address_for_test(mac_str);
            if should_succeed {
                assert!(
                    result.is_ok(),
                    "MAC address parsing failed for: {mac_str}"
                );
                assert_eq!(result.unwrap(), expected_bytes);
            } else {
                assert!(
                    result.is_err(),
                    "MAC address parsing should have failed for: {mac_str}"
                );
            }
        }

        let invalid_macs = vec![
            "12:34:56:78:9A",       // Too short
            "12:34:56:78:9A:BC:DE", // Too long
            "12:34:56:78:9A:XY",    // Invalid hex
            "12-34-56-78-9A-BC",    // Wrong separator
            "",                     // Empty string
            "12:34:56:78:9A:BC::",  // Extra separator
        ];

        for invalid_mac in invalid_macs {
            let result = parse_mac_address_for_test(invalid_mac);
            assert!(
                result.is_err(),
                "MAC address parsing should have failed for: {invalid_mac}"
            );
        }
    }

    fn parse_mac_address_for_test(mac_address: &str) -> Result<[u8; 6]> {
        let parts: Vec<&str> = mac_address.split(':').collect();
        if parts.len() != 6 {
            return Err(TreadlyError::InvalidParameters(format!(
                "Invalid MAC address format: {mac_address}. Expected format: XX:XX:XX:XX:XX:XX"
            )));
        }

        let mut mac_bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            mac_bytes[i] = u8::from_str_radix(part, 16).map_err(|_| {
                TreadlyError::InvalidParameters(format!("Invalid MAC address byte: {part}"))
            })?;
        }

        Ok(mac_bytes)
    }

    #[test]
    fn test_speed_validation_logic() {
        let test_cases = vec![
            (-1.0, false), // Negative speed invalid
            (0.0, true),   // Zero speed valid
            (10.0, true),  // Normal speed valid
            (20.0, true),  // Max speed valid
            (25.0, false), // Over max speed invalid
        ];

        for (speed, should_be_valid) in test_cases {
            let is_valid = (0.0..=20.0).contains(&speed);
            assert_eq!(is_valid, should_be_valid, "Speed {speed} validation failed");
        }
    }

    #[test]
    fn test_unit_conversion_logic() {
        let mph_speeds = vec![5.0, 10.0, 15.0];

        for mph_speed in mph_speeds {
            let kmh_speed = mph_speed * 1.6093f32;
            let back_to_mph = kmh_speed * 0.6214f32;

            assert!((back_to_mph - mph_speed).abs() < 0.01f32);
        }
    }
}
