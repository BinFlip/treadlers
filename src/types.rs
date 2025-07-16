use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

/// Speed unit for treadmill operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedUnit {
    /// Kilometers per hour
    Kilometers,
    /// Miles per hour
    Miles,
}

impl fmt::Display for SpeedUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kilometers => write!(f, "km/h"),
            Self::Miles => write!(f, "mph"),
        }
    }
}

/// Device operational mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceMode {
    /// Device is powered on but not active
    Awake = 0,
    /// Device is actively running
    Active = 1,
    /// Device is idle/paused
    Idle = 2,
    /// Unknown state
    Unknown = 3,
}

impl From<u8> for DeviceMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Awake,
            1 => Self::Active,
            2 => Self::Idle,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Awake => write!(f, "Awake"),
            Self::Active => write!(f, "Active"),
            Self::Idle => write!(f, "Idle"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Emergency stop state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyStopState {
    /// Normal operation
    Normal,
    /// Emergency stop is active
    Active,
    /// Emergency stop reset required
    ResetRequired,
}

impl fmt::Display for EmergencyStopState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Active => write!(f, "Emergency Stop Active"),
            Self::ResetRequired => write!(f, "Reset Required"),
        }
    }
}

/// Authentication status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationStatus {
    /// Not authenticated
    NotAuthenticated,
    /// Authentication in progress
    InProgress,
    /// Successfully authenticated
    Authenticated,
    /// Authentication failed
    Failed,
}

/// Temperature status codes extracted from decompiled Android app
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureStatus {
    /// Normal temperature operation
    Normal = 0,
    /// Critical temperature - forces emergency stop
    Stop = 1,
    /// High temperature - forces speed reduction
    ReduceSpeed = 2,
    /// Temperature sensor error
    Unknown = 3,
}

impl From<u8> for TemperatureStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Stop,
            2 => Self::ReduceSpeed,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for TemperatureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Stop => write!(f, "Emergency Stop - Critical Temperature"),
            Self::ReduceSpeed => write!(f, "Warning - High Temperature"),
            Self::Unknown => write!(f, "Temperature Sensor Error"),
        }
    }
}

/// Device status codes extracted from decompiled Android app
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatusCode {
    /// Normal operation
    NoError = 0,
    /// High temperature detected - safety condition
    HighTemperature = 1,
    /// `WiFi` scanning in progress
    WifiScanning = 2,
    /// Device requires power cycle - safety condition
    PowerCycleRequired = 3,
    /// Request processing error
    RequestError = 4,
    /// `WiFi` connection failed
    WifiNotConnected = 5,
    /// `WiFi` system error
    WifiError = 6,
}

impl From<u8> for DeviceStatusCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::HighTemperature,
            2 => Self::WifiScanning,
            3 => Self::PowerCycleRequired,
            5 => Self::WifiNotConnected,
            6 => Self::WifiError,
            _ => Self::RequestError,
        }
    }
}

impl fmt::Display for DeviceStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoError => write!(f, "No Error"),
            Self::HighTemperature => write!(f, "High Temperature"),
            Self::WifiScanning => write!(f, "WiFi Scanning"),
            Self::PowerCycleRequired => write!(f, "Power Cycle Required"),
            Self::RequestError => write!(f, "Request Error"),
            Self::WifiNotConnected => write!(f, "WiFi Not Connected"),
            Self::WifiError => write!(f, "WiFi Error"),
        }
    }
}

/// Connection health status for monitoring connection safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionHealth {
    /// Connection is healthy
    Healthy,
    /// Connection is degraded but functional
    Degraded,
    /// Connection is unstable
    Unstable,
    /// Connection is lost
    Lost,
}

impl fmt::Display for ConnectionHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Unstable => write!(f, "Unstable"),
            Self::Lost => write!(f, "Lost"),
        }
    }
}

/// Speed information
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpeedInfo {
    /// Current speed
    pub current: f32,
    /// Target speed
    pub target: f32,
    /// Minimum speed
    pub minimum: f32,
    /// Maximum speed
    pub maximum: f32,
    /// Speed unit
    pub unit: SpeedUnit,
}

impl SpeedInfo {
    /// Create new speed info
    #[must_use]
    pub const fn new(current: f32, target: f32, min: f32, max: f32, unit: SpeedUnit) -> Self {
        Self {
            current,
            target,
            minimum: min,
            maximum: max,
            unit,
        }
    }

    /// Convert speed to the specified unit
    #[must_use]
    pub fn convert_to(&self, unit: SpeedUnit) -> Self {
        if self.unit == unit {
            return *self;
        }

        let (current, target, min, max) = match (self.unit, unit) {
            (SpeedUnit::Kilometers, SpeedUnit::Miles) => (
                self.current * 0.6214,
                self.target * 0.6214,
                self.minimum * 0.6214,
                self.maximum * 0.6214,
            ),
            (SpeedUnit::Miles, SpeedUnit::Kilometers) => (
                self.current * 1.6093,
                self.target * 1.6093,
                self.minimum * 1.6093,
                self.maximum * 1.6093,
            ),
            _ => (self.current, self.target, self.minimum, self.maximum),
        };

        Self {
            current,
            target,
            minimum: min,
            maximum: max,
            unit,
        }
    }
}

/// Device status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceStatus {
    /// Current speed information
    pub speed: SpeedInfo,
    /// Device operational mode
    pub mode: DeviceMode,
    /// Emergency stop state
    pub emergency_stop: EmergencyStopState,
    /// Power state
    pub power_on: bool,
    /// Handrail status
    pub handrail_enabled: bool,
    /// Current distance traveled
    pub distance: f32,
    /// Step count
    pub steps: u32,
    /// Device temperature (Celsius)
    pub temperature: f32,
    /// Temperature status for safety monitoring
    pub temperature_status: TemperatureStatus,
    /// Device status code for error monitoring
    pub device_status_code: DeviceStatusCode,
    /// Connection health for safety monitoring
    pub connection_health: ConnectionHealth,
    /// Authentication status
    pub authentication: AuthenticationStatus,
    /// Session active
    pub session_active: bool,
    /// Last status update timestamp
    pub timestamp: SystemTime,
}

impl Default for DeviceStatus {
    fn default() -> Self {
        Self {
            speed: SpeedInfo::new(0.0, 0.0, 0.0, 20.0, SpeedUnit::Kilometers),
            mode: DeviceMode::Unknown,
            emergency_stop: EmergencyStopState::Normal,
            power_on: false,
            handrail_enabled: true,
            distance: 0.0,
            steps: 0,
            temperature: 0.0,
            temperature_status: TemperatureStatus::Normal,
            device_status_code: DeviceStatusCode::NoError,
            connection_health: ConnectionHealth::Healthy,
            authentication: AuthenticationStatus::NotAuthenticated,
            session_active: false,
            timestamp: SystemTime::now(),
        }
    }
}

/// Device information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name
    pub name: String,
    /// Device MAC address
    pub mac_address: Option<String>,
    /// Signal strength (RSSI)
    pub rssi: i16,
    /// Device priority (from manufacturer data)
    pub priority: i8,
    /// Firmware version
    pub firmware_version: Option<String>,
    /// Hardware version
    pub hardware_version: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
}

impl DeviceInfo {
    /// Create new device info
    #[must_use]
    pub const fn new(name: String, rssi: i16, priority: i8) -> Self {
        Self {
            name,
            mac_address: None,
            rssi,
            priority,
            firmware_version: None,
            hardware_version: None,
            serial_number: None,
        }
    }
}

/// Connection parameters
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable authentication
    pub authenticate: bool,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Scan timeout in milliseconds
    pub scan_timeout_ms: u64,
}

/// Protocol-compliant timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Default command timeout in milliseconds
    pub default_timeout_ms: u64,
    /// Authentication timeout in milliseconds
    pub auth_timeout_ms: u64,
    /// Status request timeout in milliseconds
    pub status_timeout_ms: u64,
    /// Emergency stop timeout in milliseconds
    pub emergency_stop_timeout_ms: u64,
    /// Speed command timeout in milliseconds
    pub speed_command_timeout_ms: u64,
    /// Power command timeout in milliseconds
    pub power_command_timeout_ms: u64,
    /// Secure authentication timeout in milliseconds
    pub secure_auth_timeout_ms: u64,
    /// Connection health check timeout in milliseconds
    pub connection_health_timeout_ms: u64,
    /// Maximum retry attempts for failed commands
    pub max_retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            authenticate: true,
            retry_attempts: 3,
            scan_timeout_ms: 10_000,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 3_000,
            auth_timeout_ms: 5_000,
            status_timeout_ms: 2_000,
            emergency_stop_timeout_ms: 5_000,
            speed_command_timeout_ms: 3_000,
            power_command_timeout_ms: 4_000,
            secure_auth_timeout_ms: 8_000,
            connection_health_timeout_ms: 30_000,
            max_retry_attempts: 3,
            retry_delay_ms: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_conversion() {
        let speed_km = SpeedInfo::new(10.0, 12.0, 0.0, 20.0, SpeedUnit::Kilometers);
        let speed_mph = speed_km.convert_to(SpeedUnit::Miles);

        assert!((speed_mph.current - 6.214).abs() < 0.01);
        assert_eq!(speed_mph.unit, SpeedUnit::Miles);
    }

    #[test]
    fn test_device_mode_from_u8() {
        assert_eq!(DeviceMode::from(0), DeviceMode::Awake);
        assert_eq!(DeviceMode::from(1), DeviceMode::Active);
        assert_eq!(DeviceMode::from(2), DeviceMode::Idle);
        assert_eq!(DeviceMode::from(99), DeviceMode::Unknown);
    }

    #[test]
    fn test_timeout_config_defaults() {
        let config = TimeoutConfig::default();

        assert_eq!(config.default_timeout_ms, 3_000);
        assert_eq!(config.auth_timeout_ms, 5_000);
        assert_eq!(config.status_timeout_ms, 2_000);
        assert_eq!(config.emergency_stop_timeout_ms, 5_000);
        assert_eq!(config.speed_command_timeout_ms, 3_000);
        assert_eq!(config.power_command_timeout_ms, 4_000);
        assert_eq!(config.secure_auth_timeout_ms, 8_000);
        assert_eq!(config.connection_health_timeout_ms, 30_000);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_delay_ms, 500);
    }

    #[test]
    fn test_speed_unit_conversion() {
        let speed_km = SpeedInfo::new(10.0, 12.0, 0.0, 20.0, SpeedUnit::Kilometers);
        let speed_mph = speed_km.convert_to(SpeedUnit::Miles);

        // 10 km/h should be approximately 6.214 mph
        assert!((speed_mph.current - 6.214).abs() < 0.01);
        assert!((speed_mph.target - 7.457).abs() < 0.01);
        assert_eq!(speed_mph.unit, SpeedUnit::Miles);

        // Converting back should give original values
        let speed_km_back = speed_mph.convert_to(SpeedUnit::Kilometers);
        assert!((speed_km_back.current - 10.0).abs() < 0.01);
        assert!((speed_km_back.target - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_device_info_creation() {
        let info = DeviceInfo::new("Test Treadmill".to_string(), -50, 1);
        assert_eq!(info.name, "Test Treadmill");
        assert_eq!(info.rssi, -50);
        assert_eq!(info.priority, 1);
        assert!(info.mac_address.is_none());
    }

    #[test]
    fn test_connection_params_default() {
        let params = ConnectionParams::default();
        assert_eq!(params.timeout_ms, 30_000);
        assert!(params.authenticate);
        assert_eq!(params.retry_attempts, 3);
        assert_eq!(params.scan_timeout_ms, 10_000);
    }
}
