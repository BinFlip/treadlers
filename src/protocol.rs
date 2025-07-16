use crate::{
    error::{Result, TreadlyError},
    types::{
        AuthenticationStatus, ConnectionHealth, DeviceMode, DeviceStatus, DeviceStatusCode,
        EmergencyStopState, SpeedInfo, SpeedUnit, TemperatureStatus,
    },
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::SystemTime;

/// Message size in bytes
pub const MESSAGE_SIZE: usize = 20;

/// Payload size in bytes
pub const PAYLOAD_SIZE: usize = 16;

/// Authentication secret key extracted from decompiled Treadly Android app v1.1.8
///
/// This 4-byte sequence is used for basic authentication with the treadmill.
/// The bytes [0xA6, 0x5B, 0xF2, 0x83] were discovered through reverse engineering
/// of the official application's authentication mechanism.
pub const AUTH_SECRET_KEY: [u8; 4] = [0xA6, 0x5B, 0xF2, 0x83];

/// Message IDs for treadmill commands extracted through reverse engineering
///
/// These command IDs were discovered by analyzing the decompiled Treadly Android app v1.1.8.
/// Each command corresponds to a specific treadmill function and follows a consistent
/// protocol structure with 20-byte messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageId {
    /// Power on/off the treadmill
    Power = 0x01,
    /// Increase speed incrementally
    SpeedUp = 0x02,
    /// Decrease speed incrementally
    SpeedDown = 0x03,
    /// Request current device status
    Status = 0x04,
    /// Set specific speed value
    SetSpeed = 0x0A,

    /// Set units to kilometers per hour
    SetUnitKilometers = 0x08,
    /// Set units to miles per hour
    SetUnitMiles = 0x09,

    /// Extended status request with additional info
    StatusEx = 0x0C,
    /// Get device information and version details
    GetDeviceInfo = 0x10,
    /// Subscribe to continuous status updates
    SubscribeStatus = 0x11,
    /// Extended status v2 with enhanced data
    StatusEx2 = 0x54,

    /// Basic authentication with secret key
    Authenticate = 0x13,
    /// Verify authentication response
    AuthenticateVerify = 0x14,
    /// Enhanced secure authentication
    SecureAuthenticate = 0x26,
    /// Verify secure authentication response
    SecureAuthenticateVerify = 0x27,

    /// Emergency stop command - immediate halt
    EmergencyStopRequest = 0x18,
    /// Reset from emergency stop state
    ResetStop = 0x37,
    /// Enable/disable handrail emergency stop
    SetEmergHandrailEnabled = 0x17,

    /// Configure acceleration zone start position
    SetAccelerateZoneStart = 0x15,
    /// Configure deceleration zone end position
    SetDecelerateZoneEnd = 0x16,

    /// Broadcast device status to all connected clients
    BroadcastDeviceStatus = 0x33,

    /// Enable/disable game mode functionality
    SetGameMode = 0x4E,
    /// Configure game mode display settings
    SetGameModeDisplay = 0x51,

    /// Enable/disable user interaction features
    UserInteractionSetEnable = 0x4A,
    /// Get current user interaction status
    UserInteractionStatus = 0x49,
    /// Configure user interaction step settings
    UserInteractionSteps = 0x48,
    /// Configure handrail interaction settings
    UserInteractionHandrail = 0x52,

    /// Get Bluetooth audio pairing password
    GetBtAudioPassword = 0x43,
    /// Set Bluetooth audio pairing password
    SetBtAudioPassword = 0x44,
    /// Delete all paired phone connections
    SetDeletePairedPhones = 0x55,
    /// Enable BLE functionality
    BleEnableRequest = 0x1E,

    /// Reset maintenance counters and logs
    MaintenanceResetRequest = 0x19,
    /// Manual maintenance step operation
    MaintenanceStepRequest = 0x1A,
    /// Factory reset device to defaults
    FactoryReset = 0x0B,

    /// Start diagnostic test sequence
    TestStart = 0x28,
    /// Test notification message
    TestNotification = 0x25,
    /// Start BLE remote connectivity test
    StartBleRemoteTest = 0x4F,
    /// Get BLE remote test results
    BleRemoteTestResults = 0x50,

    /// Pause treadmill operation
    Pause = 0x42,
    /// Set remote control status
    SetRemoteStatus = 0x36,
    /// Set total operation status
    SetTotalStatus = 0x35,
    /// Get device MAC address
    MacAddress = 0x2B,
    /// Verify MAC address authenticity
    VerifyMacAddress = 0x2C,

    /// Validate device authenticity
    ValidateDevice = 0x20,
    /// Diagnostic status information
    StatusDiagnostic = 0x0D,
    /// Set infrared mode
    SetIrMode = 0x2A,
    /// Get device debug log
    DeviceDebugLog = 0x39,
    /// Get infrared debug log
    DeviceIrDebugLog = 0x3A,
}

impl MessageId {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MessageId::Power),
            0x02 => Some(MessageId::SpeedUp),
            0x03 => Some(MessageId::SpeedDown),
            0x04 => Some(MessageId::Status),
            0x08 => Some(MessageId::SetUnitKilometers),
            0x09 => Some(MessageId::SetUnitMiles),
            0x0A => Some(MessageId::SetSpeed),
            0x0C => Some(MessageId::StatusEx),
            0x10 => Some(MessageId::GetDeviceInfo),
            0x11 => Some(MessageId::SubscribeStatus),
            0x13 => Some(MessageId::Authenticate),
            0x14 => Some(MessageId::AuthenticateVerify),
            0x15 => Some(MessageId::SetAccelerateZoneStart),
            0x16 => Some(MessageId::SetDecelerateZoneEnd),
            0x17 => Some(MessageId::SetEmergHandrailEnabled),
            0x18 => Some(MessageId::EmergencyStopRequest),
            0x26 => Some(MessageId::SecureAuthenticate),
            0x27 => Some(MessageId::SecureAuthenticateVerify),
            0x33 => Some(MessageId::BroadcastDeviceStatus),
            0x37 => Some(MessageId::ResetStop),
            0x43 => Some(MessageId::GetBtAudioPassword),
            0x44 => Some(MessageId::SetBtAudioPassword),
            0x49 => Some(MessageId::UserInteractionStatus),
            0x4A => Some(MessageId::UserInteractionSetEnable),
            0x4E => Some(MessageId::SetGameMode),
            0x51 => Some(MessageId::SetGameModeDisplay),
            0x54 => Some(MessageId::StatusEx2),
            0x55 => Some(MessageId::SetDeletePairedPhones),
            0x20 => Some(MessageId::ValidateDevice),
            0x0D => Some(MessageId::StatusDiagnostic),
            0x2A => Some(MessageId::SetIrMode),
            0x39 => Some(MessageId::DeviceDebugLog),
            0x3A => Some(MessageId::DeviceIrDebugLog),
            0x1E => Some(MessageId::BleEnableRequest),
            0x19 => Some(MessageId::MaintenanceResetRequest),
            0x1A => Some(MessageId::MaintenanceStepRequest),
            0x0B => Some(MessageId::FactoryReset),
            0x28 => Some(MessageId::TestStart),
            0x25 => Some(MessageId::TestNotification),
            0x4F => Some(MessageId::StartBleRemoteTest),
            0x50 => Some(MessageId::BleRemoteTestResults),
            0x42 => Some(MessageId::Pause),
            0x36 => Some(MessageId::SetRemoteStatus),
            0x35 => Some(MessageId::SetTotalStatus),
            0x2B => Some(MessageId::MacAddress),
            0x2C => Some(MessageId::VerifyMacAddress),
            0x48 => Some(MessageId::UserInteractionSteps),
            0x52 => Some(MessageId::UserInteractionHandrail),
            _ => None,
        }
    }
}

/// Status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StatusCode {
    /// Standard request/success status
    Request = 0x00,
    /// Secure authentication required before proceeding
    SecureAuthRequired = 0x81,
}

/// Success status code indicating successful operation
pub const STATUS_SUCCESS: u8 = 0x00;

/// Treadmill protocol message based on reverse engineered communication format
///
/// The message structure was discovered through analysis of the Treadly Android app:
/// - Fixed 20-byte message size with specific byte layout
/// - Command ID repeated in bytes 0 and 2 for validation
/// - Status/response code in byte 1  
/// - 16-byte payload starting at byte 3
/// - Little-endian encoding for numeric values
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message ID
    pub id: MessageId,
    /// Status code
    pub status: u8,
    /// Payload data
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(id: MessageId, payload: Vec<u8>) -> Self {
        Self {
            id,
            status: StatusCode::Request as u8,
            payload,
        }
    }

    /// Create a command message with empty payload
    pub fn command(id: MessageId) -> Self {
        Self::new(id, vec![0; PAYLOAD_SIZE])
    }

    /// Create a speed command message
    pub fn set_speed(speed: f32) -> Self {
        let mut payload = vec![0; PAYLOAD_SIZE];
        let speed_bytes = (speed * 10.0) as u32;
        payload[0..4].copy_from_slice(&speed_bytes.to_le_bytes());
        Self::new(MessageId::SetSpeed, payload)
    }

    /// Create authentication message
    pub fn authenticate() -> Self {
        let mut payload = vec![0; PAYLOAD_SIZE];
        payload[0..4].copy_from_slice(&AUTH_SECRET_KEY);
        Self::new(MessageId::Authenticate, payload)
    }

    /// Create handrail enable/disable message
    pub fn set_handrail_enabled(enabled: bool) -> Self {
        let mut payload = vec![0; PAYLOAD_SIZE];
        payload[0] = if enabled { 1 } else { 0 };
        Self::new(MessageId::SetEmergHandrailEnabled, payload)
    }

    /// Create secure authentication verify message with MD5 hash
    ///
    /// This message is used in the second step of secure authentication
    /// to send the computed MD5 hash response back to the device.
    ///
    /// # Arguments
    ///
    /// * `hash_response` - 16-byte MD5 hash computed from challenge + secret key
    pub fn secure_authenticate_verify(hash_response: [u8; 16]) -> Self {
        let mut payload = vec![0; PAYLOAD_SIZE];
        payload[0..16].copy_from_slice(&hash_response);
        Self::new(MessageId::SecureAuthenticateVerify, payload)
    }

    /// Create MAC address verification message
    ///
    /// This message is used to verify the authenticity of the device's MAC address
    /// by sending the expected MAC address bytes to the device for validation.
    ///
    /// # Arguments
    ///
    /// * `mac_bytes` - 6-byte MAC address to verify
    pub fn verify_mac_address(mac_bytes: [u8; 6]) -> Self {
        let mut payload = vec![0; PAYLOAD_SIZE];
        payload[0..6].copy_from_slice(&mac_bytes);
        Self::new(MessageId::VerifyMacAddress, payload)
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(MESSAGE_SIZE);

        buf.put_u8(self.id as u8);

        buf.put_u8(self.status);

        buf.put_u8(self.id as u8);

        let payload_len = std::cmp::min(self.payload.len(), PAYLOAD_SIZE);
        buf.extend_from_slice(&self.payload[..payload_len]);

        while buf.len() < MESSAGE_SIZE - 1 {
            buf.put_u8(0);
        }

        buf.put_u8(0);

        buf.freeze()
    }

    /// Parse message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < MESSAGE_SIZE {
            return Err(TreadlyError::ParseError(format!(
                "Message too short: {} bytes, expected {}",
                data.len(),
                MESSAGE_SIZE
            )));
        }

        let mut buf = data;

        let cmd = buf.get_u8();
        let status = buf.get_u8();
        let id = buf.get_u8();

        if cmd != id {
            return Err(TreadlyError::ParseError(format!(
                "Command byte ({cmd:02X}) doesn't match ID byte ({id:02X})"
            )));
        }

        let message_id = MessageId::from_u8(id)
            .ok_or_else(|| TreadlyError::ParseError(format!("Unknown message ID: {id:02X}")))?;

        let payload = buf[..PAYLOAD_SIZE].to_vec();

        Ok(Self {
            id: message_id,
            status,
            payload,
        })
    }
}

/// Parse device status from message payload using reverse engineered format
///
/// Status message structure discovered from Android app analysis:
/// - Bytes 0-3: Current speed (u32 little-endian, divide by 10.0 for km/h)
/// - Bytes 4-7: Target speed (u32 little-endian, divide by 10.0 for km/h)  
/// - Byte 8: Device mode (0=Awake, 1=Active, 2=Idle, 3=Unknown)
/// - Byte 9: Status flags (bit 0=emergency stop, bit 1=reset required, bit 2=power, bit 3=handrail, bit 4=session)
/// - Bytes 10-13: Distance (u32 little-endian, divide by 100.0 for units)
/// - Bytes 14-17: Steps (u32 little-endian)
/// - Bytes 18-19: Temperature (u16 little-endian, divide by 10.0 for Celsius)
/// - Byte 20: Temperature status code (0=Normal, 1=Stop, 2=ReduceSpeed, 3=Unknown)
/// - Byte 21: Device status code (0=NoError, 1=HighTemperature, 2=WifiScanning, etc.)
pub fn parse_device_status(message: &Message) -> Result<DeviceStatus> {
    if message.payload.len() < 13 {
        return Err(TreadlyError::ParseError(
            "Status payload too short".to_string(),
        ));
    }

    let mut buf = &message.payload[..];

    let current_speed = buf.get_u32_le() as f32 / 10.0;
    let target_speed = buf.get_u32_le() as f32 / 10.0;

    let mode_byte = buf.get_u8();
    let mode = DeviceMode::from(mode_byte);

    let flags = buf.get_u8();
    let emergency_stop = if flags & 0x01 != 0 {
        EmergencyStopState::Active
    } else if flags & 0x02 != 0 {
        EmergencyStopState::ResetRequired
    } else {
        EmergencyStopState::Normal
    };

    let power_on = flags & 0x04 != 0;
    let handrail_enabled = flags & 0x08 != 0;
    let session_active = flags & 0x10 != 0;

    let distance = if buf.remaining() >= 4 {
        buf.get_u32_le() as f32 / 100.0
    } else {
        0.0
    };

    let steps = if buf.remaining() >= 4 {
        buf.get_u32_le()
    } else {
        0
    };

    let temperature = if buf.remaining() >= 2 {
        buf.get_u16_le() as f32 / 10.0
    } else {
        0.0
    };

    let temperature_status = if buf.remaining() >= 1 {
        TemperatureStatus::from(buf.get_u8())
    } else {
        TemperatureStatus::Normal
    };

    let device_status_code = if buf.remaining() >= 1 {
        DeviceStatusCode::from(buf.get_u8())
    } else {
        DeviceStatusCode::NoError
    };

    let speed = SpeedInfo::new(
        current_speed,
        target_speed,
        0.0,
        20.0,                  // Default max speed
        SpeedUnit::Kilometers, // Default unit
    );

    Ok(DeviceStatus {
        speed,
        mode,
        emergency_stop,
        power_on,
        handrail_enabled,
        distance,
        steps,
        temperature,
        temperature_status,
        device_status_code,
        connection_health: ConnectionHealth::Healthy, // Will be updated by connection monitoring
        authentication: AuthenticationStatus::NotAuthenticated, // Will be updated separately
        session_active,
        timestamp: SystemTime::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::command(MessageId::Power);
        let bytes = msg.to_bytes();

        assert_eq!(bytes.len(), MESSAGE_SIZE);
        assert_eq!(bytes[0], MessageId::Power as u8);
        assert_eq!(bytes[1], StatusCode::Request as u8);
        assert_eq!(bytes[2], MessageId::Power as u8);
    }

    #[test]
    fn test_message_deserialization() {
        let original = Message::command(MessageId::Status);
        let bytes = original.to_bytes();
        let parsed = Message::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.status, original.status);
    }

    #[test]
    fn test_speed_message() {
        let msg = Message::set_speed(5.5);
        let bytes = msg.to_bytes();

        assert_eq!(&bytes[3..7], &55u32.to_le_bytes());
    }

    #[test]
    fn test_authentication_message() {
        let msg = Message::authenticate();
        assert_eq!(&msg.payload[0..4], &AUTH_SECRET_KEY);
    }

    #[test]
    fn test_secure_authentication_verify_message() {
        let hash = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ];
        let msg = Message::secure_authenticate_verify(hash);

        assert_eq!(msg.id, MessageId::SecureAuthenticateVerify);
        assert_eq!(&msg.payload[0..16], &hash);
    }

    #[test]
    fn test_mac_address_verification_message() {
        let mac_bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        let msg = Message::verify_mac_address(mac_bytes);

        assert_eq!(msg.id, MessageId::VerifyMacAddress);
        assert_eq!(&msg.payload[0..6], &mac_bytes);
    }

    #[test]
    fn test_handrail_message() {
        let enabled_msg = Message::set_handrail_enabled(true);
        assert_eq!(enabled_msg.payload[0], 1);

        let disabled_msg = Message::set_handrail_enabled(false);
        assert_eq!(disabled_msg.payload[0], 0);
    }
}
