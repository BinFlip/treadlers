use thiserror::Error;

/// Errors that can occur when working with Treadly treadmills
#[derive(Error, Debug)]
pub enum TreadlyError {
    /// Bluetooth Low Energy related errors
    #[error("BLE error: {0}")]
    Ble(#[from] btleplug::Error),

    /// Device not found during scanning
    #[error("Treadly device not found")]
    DeviceNotFound,

    /// Device connection failed
    #[error("Failed to connect to device: {0}")]
    ConnectionFailed(String),

    /// Device disconnected unexpectedly
    #[error("Device disconnected")]
    Disconnected,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Command timeout
    #[error("Command timed out after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Invalid command parameters
    #[error("Invalid command parameters: {0}")]
    InvalidParameters(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Device returned an error status
    #[error("Device error: status code {status:02X}")]
    DeviceError {
        /// Device status error code
        status: u8,
    },

    /// Emergency stop is active
    #[error("Emergency stop is active - reset required")]
    EmergencyStop,

    /// Device is not ready for commands
    #[error("Device not ready: {reason}")]
    NotReady {
        /// Reason why device is not ready
        reason: String,
    },

    /// Invalid device state
    #[error("Invalid device state: {state}")]
    InvalidState {
        /// Current invalid state description
        state: String,
    },

    /// Temperature safety stop activated
    #[error("Temperature safety stop activated - critical temperature reached")]
    TemperatureSafetyStop,

    /// High temperature requires speed reduction
    #[error("High temperature detected - speed reduction required")]
    HighTemperatureSpeedReduction,

    /// Temperature sensor error
    #[error("Temperature sensor error - unable to monitor safety")]
    TemperatureSensorError,

    /// Device requires power cycle
    #[error("Device requires power cycle - safety condition")]
    PowerCycleRequired,

    /// Connection lost - emergency stop activated
    #[error("Connection lost - emergency stop activated for safety")]
    ConnectionLostEmergencyStop,

    /// Connection health degraded
    #[error("Connection health degraded - monitoring for safety")]
    ConnectionHealthDegraded,

    /// Emergency stop acknowledgment not received
    #[error("Emergency stop command sent but acknowledgment not received")]
    EmergencyStopNotAcknowledged,

    /// Message parsing failed
    #[error("Failed to parse message: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for Treadly operations
pub type Result<T> = std::result::Result<T, TreadlyError>;

impl TreadlyError {
    /// Check if this error indicates a connection issue
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::Ble(_)
                | Self::ConnectionFailed(_)
                | Self::Disconnected
                | Self::DeviceNotFound
        )
    }

    /// Check if this error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::NotReady { .. }
                | Self::InvalidParameters(_)
        )
    }

    /// Check if this error requires emergency stop reset
    #[must_use]
    pub const fn requires_reset(&self) -> bool {
        matches!(self, Self::EmergencyStop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let connection_error = TreadlyError::ConnectionFailed("test".to_string());
        assert!(connection_error.is_connection_error());
        assert!(!connection_error.is_recoverable());
        assert!(!connection_error.requires_reset());

        let timeout_error = TreadlyError::Timeout { timeout_ms: 5000 };
        assert!(!timeout_error.is_connection_error());
        assert!(timeout_error.is_recoverable());
        assert!(!timeout_error.requires_reset());

        let emergency_error = TreadlyError::EmergencyStop;
        assert!(!emergency_error.is_connection_error());
        assert!(!emergency_error.is_recoverable());
        assert!(emergency_error.requires_reset());
    }

    #[test]
    fn test_error_display() {
        let error = TreadlyError::InvalidParameters("speed out of range".to_string());
        let error_string = format!("{error}");
        assert!(error_string.contains("Invalid command parameters"));
        assert!(error_string.contains("speed out of range"));
    }
}
