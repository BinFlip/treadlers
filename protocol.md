# Treadly Treadmill Bluetooth Communication Protocol Documentation

## Overview

This document provides a analysis of the Treadly treadmill's Bluetooth Low Energy (BLE) communication protocol, extracted from the decompiled Android application (version 1.1.8). This protocol can be used to create alternative applications or libraries to control Treadly treadmills.

**Context**: Treadly shut down as a company, leaving their treadmills as "dead bricks" when their backend services went offline. This protocol documentation was created through extensive reverse engineering of the official Android application to restore functionality and enable custom implementations.

## Bluetooth Low Energy (BLE) Configuration

### Service and Characteristic UUIDs

The Treadly treadmill uses a custom BLE service based on Nordic UART Service (NUS):

- **Primary Service UUID**: `6E400000-B5A3-F393-E0A9-E50E24DCCA9E`
- **TX Characteristic UUID**: `6E400003-B5A3-F393-E0A9-E50E24DCCA9E` (notifications from device)
- **RX Characteristic UUID**: `6E400002-B5A3-F393-E0A9-E50E24DCCA9E` (write to device)
- **Service UUID Mask**: `11110000-1111-1111-1111-111111111111`

### Device Discovery

- **Manufacturer ID**: `0x59` (89 decimal)
- **Scan Mode**: High Performance (mode 2)
- **Callback Type**: First Match (type 1)
- **Device Name Pattern**: Contains "Treadly" (case-insensitive)
- **Signal Strength**: RSSI values used for device prioritization

## Message Protocol Structure

### Basic Message Format

All messages follow a strict 20-byte format:

- **Message Size**: 20 bytes (0x14)
- **Payload Size**: 16 bytes (0x10)
- **Structure**:
  - Byte 0: Command ID
  - Byte 1: Status/Response Code
  - Byte 2: Message ID (duplicate of Command ID for validation)
  - Bytes 3-18: Payload (16 bytes)
  - Byte 19: Padding/checksum (typically 0x00)

### Status Codes

- **REQUEST**: `0x00` - Standard request/success status
- **SUCCESS**: `0x00` - Operation completed successfully
- **SECURE_AUTHENTICATION_REQUIRED**: `0x81` - Device requires secure authentication

### Constants and Conversion Factors

- **DISTANCE_TO_FLOAT**: 100.0f (distance values divided by 100 for display)
- **SPEED_TO_FLOAT**: 10.0f (speed values divided by 10 for display)
- **TEMP_TO_FLOAT**: 10.0f (temperature values divided by 10 for display)
- **MILES_TO_KM_CONVERT**: 1.6093f
- **KM_TO_MILES_CONVERT**: 0.6214f

## Complete Command Protocol

### Core Control Commands

#### Power Control

- **MESSAGE_ID_POWER**: `0x01` - Power on/off the treadmill

#### Speed Control

- **MESSAGE_ID_SPEED_UP**: `0x02` - Incrementally increase speed
- **MESSAGE_ID_SPEED_DOWN**: `0x03` - Incrementally decrease speed
- **MESSAGE_ID_SET_SPEED**: `0x0A` - Set specific speed value

#### Emergency and Safety

- **MESSAGE_ID_EMERGENCY_STOP_REQUEST**: `0x18` - **CRITICAL**: Emergency stop (immediate halt)
- **MESSAGE_ID_RESET_STOP**: `0x37` - Reset from emergency stop state
- **MESSAGE_ID_SET_EMERG_HANDRAIL_ENABLED**: `0x17` - Enable/disable handrail emergency stop
- **MESSAGE_ID_PAUSE**: `0x42` - Pause treadmill operation

#### Status and Information

- **MESSAGE_ID_STATUS**: `0x04` - Request current status
- **MESSAGE_ID_STATUS_EX**: `0x0C` - Extended status request
- **MESSAGE_ID_STATUS_EX_2**: `0x54` - Extended status v2 with enhanced data
- **MESSAGE_ID_STATUS_DIAGNOSTIC**: `0x0D` - Diagnostic status information
- **MESSAGE_ID_GET_DEVICE_INFO**: `0x10` - Get device information and version
- **MESSAGE_ID_SUBSCRIBE_STATUS**: `0x11` - Subscribe to continuous status updates
- **MESSAGE_ID_BROADCAST_DEVICE_STATUS**: `0x33` - Broadcast device status to all clients

#### Configuration

- **MESSAGE_ID_SET_UNIT_KILOMETERS**: `0x08` - Set units to kilometers
- **MESSAGE_ID_SET_UNIT_MILES**: `0x09` - Set units to miles
- **MESSAGE_ID_SET_ACCELERATE_ZONE_START**: `0x15` - Configure acceleration zones
- **MESSAGE_ID_SET_DECELERATE_ZONE_END**: `0x16` - Configure deceleration zones

### Authentication and Security

#### Authentication Commands

- **MESSAGE_ID_AUTHENTICATE**: `0x13` - Standard authentication
- **MESSAGE_ID_AUTHENTICATE_VERIFY**: `0x14` - Verify authentication response
- **MESSAGE_ID_SECURE_AUTHENTICATE**: `0x26` - Enhanced secure authentication
- **MESSAGE_ID_SECURE_AUTHENTICATE_VERIFY**: `0x27` - Verify secure authentication

#### Authentication Process

1. **Standard Authentication**: Uses 4-byte secret key `[0xA6, 0x5B, 0xF2, 0x83]`
2. **Enhanced Authentication**: Uses longer key `[0x54, 0x72, 0x65, 0x61, 0x64, 0x6c, 0x79, 0x4f, 0x6e, 0x6c, 0x79]` ("TreadlyOnly")
3. **MD5 Challenge-Response**: Device sends challenge, app responds with MD5 hash

### Advanced Commands

#### Maintenance and Testing

- **MESSAGE_ID_MAINTENANCE_RESET_REQUEST**: `0x19` - Reset maintenance counters
- **MESSAGE_ID_MAINTENANCE_STEP_REQUEST**: `0x1A` - Manual maintenance step
- **MESSAGE_ID_FACTORY_RESET**: `0x0B` - Factory reset device to defaults
- **MESSAGE_ID_TEST_START**: `0x28` - Start diagnostic test sequence
- **MESSAGE_ID_TEST_NOTIFICATION**: `0x25` - Test notification message

#### Game Mode and Interactive Features

- **MESSAGE_ID_SET_GAME_MODE**: `0x4E` - Enable/disable game mode functionality
- **MESSAGE_ID_SET_GAME_MODE_DISPLAY**: `0x51` - Configure game mode display settings
- **MESSAGE_ID_USER_INTERACTION_SET_ENABLE**: `0x4A` - Enable user interaction features
- **MESSAGE_ID_USER_INTERACTION_STATUS**: `0x49` - Get current interaction status
- **MESSAGE_ID_USER_INTERACTION_STEPS**: `0x48` - Configure interaction step settings
- **MESSAGE_ID_USER_INTERACTION_HANDRAIL**: `0x52` - Configure handrail interaction

#### Bluetooth Audio and Connectivity

- **MESSAGE_ID_GET_BT_AUDIO_PASSWORD**: `0x43` - Get Bluetooth audio pairing password
- **MESSAGE_ID_SET_BT_AUDIO_PASSWORD**: `0x44` - Set Bluetooth audio pairing password
- **MESSAGE_ID_SET_DELETE_PAIRED_PHONES**: `0x55` - Delete all paired phone connections
- **MESSAGE_ID_BLE_ENABLE_REQUEST**: `0x1E` - Enable BLE functionality

#### Network and Device Management

- **MESSAGE_ID_VALIDATE_DEVICE**: `0x20` - Validate device authenticity
- **MESSAGE_ID_MAC_ADDRESS**: `0x2B` - Get device MAC address
- **MESSAGE_ID_VERIFY_MAC_ADDRESS**: `0x2C` - Verify MAC address authenticity
- **MESSAGE_ID_SET_IR_MODE**: `0x2A` - Set infrared mode
- **MESSAGE_ID_SET_REMOTE_STATUS**: `0x36` - Set remote control status
- **MESSAGE_ID_SET_TOTAL_STATUS**: `0x35` - Set total operation status

#### BLE Testing and Diagnostics

- **MESSAGE_ID_START_BLE_REMOTE_TEST**: `0x4F` - Start BLE remote connectivity test
- **MESSAGE_ID_BLE_REMOTE_TEST_RESULTS**: `0x50` - Get BLE remote test results
- **MESSAGE_ID_DEVICE_DEBUG_LOG**: `0x39` - Get device debug log
- **MESSAGE_ID_DEVICE_IR_DEBUG_LOG**: `0x3A` - Get infrared debug log

## Device Status Information

### Device Status Codes

- **NO_ERROR**: 0 - Normal operation
- **HIGH_TEMPERATURE**: 1 - **SAFETY**: High temperature detected
- **WIFI_SCANNING**: 2 - WiFi scanning in progress
- **POWER_CYCLE_REQUIRED**: 3 - **SAFETY**: Device requires power cycle
- **REQUEST_ERROR**: 4 - Request processing error
- **WIFI_NOT_CONNECTED_ERROR**: 5 - WiFi connection failed
- **WIFI_ERROR**: 6 - WiFi system error

### Temperature Status Codes

- **TEMP_NO_ERROR**: 0 - Normal temperature
- **TEMP_STATUS_STOP**: 1 - **CRITICAL**: Temperature forces treadmill stop
- **TEMP_STATUS_REDUCE_SPEED**: 2 - **WARNING**: Temperature forces speed reduction
- **TEMP_STATUS_UNKNOWN**: 3 - Temperature sensor error

### Device Modes

- **AWAKE** (0): Device powered on but not active
- **ACTIVE** (1): Device actively running
- **IDLE** (2): Device idle/paused
- **UNKNOWN** (3): Unknown state

### Status Message Structure

Device status messages include:

- **Speed Information**: Current, target, minimum, maximum speeds with units
- **Device Mode**: Current operational state
- **Emergency Stop State**: Normal, active, or reset required
- **Temperature**: Current temperature with safety status
- **Power State**: On/off status
- **Handrail Status**: Enabled/disabled state
- **Distance and Steps**: Accumulated activity data
- **Authentication Status**: Current authentication state
- **Session Information**: Active session ownership
- **Maintenance Data**: Counters and diagnostic information

## Connection and Communication

### Connection Timeouts

- **Connection Timeout**: 30 seconds (0x7530 ms)
- **Device Active Reconnect**: 2 seconds (0x7D0 ms)
- **Message Response Timeout**: 5 seconds (typical)

### Connection Procedure

#### 1. Device Discovery

```text
1. Start BLE scan with service UUID filter: 6E400000-B5A3-F393-E0A9-E50E24DCCA9E
2. Filter devices with manufacturer ID 0x59
3. Check device name contains "Treadly" (case-insensitive)
4. Prioritize by signal strength (RSSI) and manufacturer data priority
```

#### 2. Connection Establishment

```text
1. Connect to discovered device within 30-second timeout
2. Discover services and characteristics
3. Validate service UUID and characteristic UUIDs
4. Enable notifications on TX characteristic (6E400003-B5A3-F393-E0A9-E50E24DCCA9E)
5. Prepare to write commands to RX characteristic (6E400002-B5A3-F393-E0A9-E50E24DCCA9E)
```

#### 3. Authentication (if required)

```text
1. Send MESSAGE_ID_AUTHENTICATE (0x13) with secret key
2. Handle authentication challenge/response
3. Process MD5 hash challenge if secure authentication required
4. Verify with MESSAGE_ID_AUTHENTICATE_VERIFY (0x14)
5. Handle SECURE_AUTHENTICATION_REQUIRED (0x81) if needed
```

#### 4. Device Activation

```text
1. Send MESSAGE_ID_VALIDATE_DEVICE (0x20) for device validation
2. Configure device settings (units, zones, etc.)
3. Send MESSAGE_ID_SUBSCRIBE_STATUS (0x11) for continuous updates
4. Begin normal operation with status monitoring
```

### Error Recovery and Reconnection

#### Automatic Reconnection

- **Connection Monitoring**: Continuous connection state tracking
- **Automatic Retry**: Timer-based reconnection attempts
- **Validation**: Re-authenticate and validate after reconnection
- **State Restoration**: Restore previous settings and subscriptions

#### Error Handling Strategies

- **Connection Failures**: Retry with exponential backoff
- **Authentication Failures**: Clear stored credentials and re-authenticate
- **Device Disconnection**: Immediate reconnection attempt
- **Command Timeouts**: Resend command with status check
- **Protocol Errors**: Reset connection and restart handshake

## Safety Systems

### Emergency Stop System

The treadmill implements multiple layers of emergency stop protection:

#### Emergency Stop Triggers

1. **Manual Emergency Stop**: MESSAGE_ID_EMERGENCY_STOP_REQUEST (0x18)
2. **Handrail Emergency Stop**: Automatic trigger from handrail sensor
3. **Temperature Emergency Stop**: Automatic trigger from temperature monitoring
4. **Connection Loss**: Automatic stop if control connection is lost

#### Emergency Stop States

- **Normal Operation**: No emergency conditions
- **Emergency Stop Active**: Treadmill immediately halted
- **Reset Required**: Manual reset needed to resume operation

#### Emergency Stop Recovery

```text
1. Identify emergency stop cause
2. Address underlying issue (temperature, handrail, etc.)
3. Send MESSAGE_ID_RESET_STOP (0x37) to clear emergency state
4. Verify normal operation before resuming
```

### Temperature Safety System

The treadmill implements multi-tiered temperature protection:

#### Temperature Monitoring

- **Continuous Monitoring**: Real-time temperature sensing
- **Safety Thresholds**: Multiple temperature levels trigger different responses
- **Automatic Protection**: No user intervention required for safety responses

#### Temperature Responses

1. **Normal Operation**: Temperature within safe range
2. **Speed Reduction**: Automatic speed reduction when temperature rises
3. **Emergency Stop**: Complete shutdown when temperature becomes critical
4. **Cool-down Period**: Enforced cooling before operation resumes

### Handrail Safety System

- **Configurable Detection**: Enable/disable handrail emergency stop
- **Real-time Monitoring**: Continuous handrail grip detection
- **Immediate Response**: Instant treadmill stop when handrail released
- **Integration**: Linked with main emergency stop system

### Speed Control Safety

- **Speed Zones**: Configurable acceleration/deceleration zones
- **Gradual Changes**: Prevented sudden speed changes
- **Maximum Limits**: Enforced maximum speed limits
- **Emergency Override**: Emergency stop overrides all speed commands

## Implementation Guide for Rust Library

### Critical Safety Requirements

#### 1. Emergency Stop Implementation

```rust
// Emergency stop must be implemented with highest priority
async fn emergency_stop(&self) -> Result<()> {
    // Send emergency stop command immediately
    let message = Message::command(MessageId::EmergencyStopRequest);
    self.send_command_priority(&message).await?;
    
    // Verify emergency stop was received
    self.verify_emergency_stop().await?;
    
    // Update internal state
    self.set_emergency_state(true).await;
    
    Ok(())
}
```

#### 2. Temperature Monitoring

```rust
// Monitor temperature status and respond automatically
async fn monitor_temperature(&self, status: &DeviceStatus) -> Result<()> {
    match status.temperature_status {
        TemperatureStatus::Stop => {
            // Force emergency stop
            self.emergency_stop().await?;
        }
        TemperatureStatus::ReduceSpeed => {
            // Automatically reduce speed
            self.reduce_speed_for_temperature().await?;
        }
        TemperatureStatus::Normal => {
            // Normal operation
        }
    }
    Ok(())
}
```

#### 3. Connection State Management

```rust
// Implement robust connection monitoring
async fn monitor_connection(&self) -> Result<()> {
    while self.should_monitor() {
        if !self.is_connected().await {
            // Attempt reconnection
            self.reconnect().await?;
            // Re-establish state
            self.restore_state().await?;
        }
        
        // Check for timeout
        if self.last_message_time().elapsed() > TIMEOUT_THRESHOLD {
            // Trigger emergency stop if no communication
            self.emergency_stop().await?;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}
```

### Recommended Architecture

#### 1. BLE Manager

- **Connection Management**: Handle discovery, connection, and disconnection
- **Message Transport**: Low-level message sending and receiving
- **Error Recovery**: Automatic reconnection and error handling
- **Notification Handling**: Process incoming device notifications

#### 2. Protocol Handler

- **Message Encoding**: Convert commands to protocol messages
- **Message Decoding**: Parse incoming status and response messages
- **Validation**: Verify message integrity and format
- **State Tracking**: Maintain device state consistency

#### 3. Safety Manager

- **Emergency Stop**: Immediate emergency stop capability
- **Temperature Monitoring**: Continuous temperature safety checking
- **Connection Monitoring**: Ensure communication reliability
- **Safety Interlocks**: Prevent dangerous operations

#### 4. Command Interface

- **High-level API**: User-friendly control interface
- **Parameter Validation**: Verify command parameters before sending
- **State Verification**: Ensure device is ready for commands
- **Error Handling**: Comprehensive error reporting and recovery

### Key Implementation Considerations

#### Safety First

- **Emergency Stop Priority**: Emergency stop commands must have highest priority
- **Connection Monitoring**: Continuous monitoring of connection state
- **Temperature Monitoring**: Automatic response to temperature conditions
- **Input Validation**: Validate all parameters before sending commands
- **State Consistency**: Maintain accurate device state tracking

#### Protocol Compliance

- **Message Format**: Strict adherence to 20-byte message format
- **Command Validation**: Verify command ID duplication in bytes 0 and 2
- **Status Handling**: Proper interpretation of status and error codes
- **Authentication**: Implement secure authentication flow
- **Timing**: Respect timeout values and retry logic

#### Error Handling

- **Connection Errors**: Robust reconnection and retry mechanisms
- **Protocol Errors**: Graceful handling of malformed messages
- **Device Errors**: Proper interpretation and response to device error codes
- **Timeout Handling**: Appropriate timeout values for all operations
- **Safety Fallbacks**: Emergency stop on critical errors

### Testing and Validation

#### Safety Testing

- **Emergency Stop Response**: Verify immediate stop on emergency command
- **Temperature Response**: Test automatic temperature safety responses
- **Connection Loss**: Verify behavior when connection is lost
- **Invalid Commands**: Test handling of invalid or malformed commands

#### Protocol Testing

- **Message Format**: Verify correct message structure and encoding
- **Authentication**: Test authentication flow and error handling
- **Status Parsing**: Validate status message interpretation
- **Command Responses**: Verify proper handling of command responses

#### Performance Testing

- **Connection Time**: Measure connection establishment time
- **Command Latency**: Test response time for critical commands
- **Reconnection Speed**: Verify reconnection performance
- **Memory Usage**: Monitor memory usage during operation

## Sample Message Construction

### Basic Speed Control

```text
To set speed to 5.0 km/h:
- Byte 0: 0x0A (MESSAGE_ID_SET_SPEED)
- Byte 1: 0x00 (REQUEST)
- Byte 2: 0x0A (Message ID repeated for validation)
- Bytes 3-6: Speed value (5.0 * 10.0 = 50.0 as u32 little-endian)
- Bytes 7-18: Padding (typically zeros)
- Byte 19: 0x00 (padding/checksum)
```

### Emergency Stop

```text
CRITICAL - Emergency stop command:
- Byte 0: 0x18 (MESSAGE_ID_EMERGENCY_STOP_REQUEST)
- Byte 1: 0x00 (REQUEST)
- Byte 2: 0x18 (Message ID repeated for validation)
- Bytes 3-19: Padding (typically zeros)
```

### Authentication

```text
Standard authentication:
- Byte 0: 0x13 (MESSAGE_ID_AUTHENTICATE)
- Byte 1: 0x00 (REQUEST)
- Byte 2: 0x13 (Message ID repeated for validation)
- Bytes 3-6: Secret key [0xA6, 0x5B, 0xF2, 0x83]
- Bytes 7-18: Padding (typically zeros)
- Byte 19: 0x00 (padding/checksum)
```

## Legacy Backend Services (Non-operational)

### Network Endpoints

The original Treadly app connected to these services (no longer operational):

- **Main API**: `https://env-staging.treadly.co:7001/api`
- **MQTT Server**: `env-staging.treadly.co:8883`
- **MQTT Credentials**: Username: "neo", Password: "fsd8*d"

**Note**: These endpoints are provided for historical context only. The Treadly backend infrastructure was shut down when the company ceased operations.

## Conclusion

This protocol documentation provides a comprehensive foundation for implementing safe and reliable Treadly treadmill control software. The emphasis on safety systems, proper error handling, and protocol compliance ensures that implementations can provide both functionality and user safety.

**Critical Safety Reminder**: Always implement emergency stop functionality as the highest priority feature. Temperature monitoring and connection state management are essential for safe operation. Test all safety systems thoroughly before deployment.

The reverse engineering process has revealed a well-designed protocol with multiple safety layers, making it possible to create robust control applications that maintain the safety standards expected from exercise equipment.
