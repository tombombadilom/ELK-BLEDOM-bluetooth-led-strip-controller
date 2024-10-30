// Import required dependencies from btleplug for Bluetooth LE functionality
use btleplug::api::{Central, Manager, Peripheral, WriteType, ScanFilter, Characteristic};
use btleplug::platform::{Manager as PlatformManager, Peripheral as PlatformPeripheral};
use std::time::Duration;

// Main struct representing a Bluetooth LED device
pub struct BleLedDevice {
    peripheral: PlatformPeripheral,
    write_characteristic: Characteristic,
}

impl BleLedDevice {
    // Creates a new BleLedDevice instance by connecting to a device with the given address
    pub async fn new_with_address(addr_str: &str) -> Result<Self, String> {
        let manager = PlatformManager::new().await
            .map_err(|e| format!("Failed to create manager: {}", e))?;

        let adapters = manager.adapters().await
            .map_err(|e| format!("Failed to get adapters: {}", e))?;

        let central = adapters.into_iter().next()
            .ok_or("No Bluetooth adapters found")?;

        // First try to find already paired device
        let peripherals = central.peripherals().await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        println!("Found {} peripherals before scan", peripherals.len());
        for p in &peripherals {
            println!("Device: {:?}", p.address());
        }

        let peripheral = match peripherals.into_iter().find(|p| p.address().to_string() == addr_str) {
            Some(p) => {
                println!("Found paired device: {:?}", p.address());
                p
            },
            None => {
                // If not found, start scanning
                println!("Device not paired, starting scan...");
                central.start_scan(ScanFilter::default()).await
                    .map_err(|e| format!("Failed to start scan: {}", e))?;

                println!("Scanning for 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;

                let peripherals = central.peripherals().await
                    .map_err(|e| format!("Failed to get peripherals: {}", e))?;

                println!("Found {} peripherals after scan", peripherals.len());
                for p in &peripherals {
                    // Check if device is in pairing mode
                    if let Ok(Some(props)) = p.properties().await {
                        println!("Device: {:?} - Name: {} - RSSI: {:?}", 
                            p.address(), 
                            props.local_name.unwrap_or_default(),
                            props.rssi
                        );
                    }
                }

                let p = peripherals.into_iter()
                    .find(|p| p.address().to_string() == addr_str)
                    .ok_or("Device not found")?;

                // Verify if device is in pairing mode
                if let Ok(Some(props)) = p.properties().await {
                    if props.rssi.unwrap_or(-100) < -90 {
                        return Err("Device signal too weak or not in pairing mode. Please reset the device and try again.".to_string());
                    }
                    println!("Device found with good signal strength!");
                }

                println!("Found device in scan: {:?}", p.address());
                let _ = central.stop_scan().await;
                p
            }
        };

        // Connection needed
        println!("Connecting to device...");
        for attempt in 1..=3 {
            println!("Connection attempt {} of 3...", attempt);
            
            // Essayons de se déconnecter d'abord au cas où
            let _ = peripheral.disconnect().await;
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            match peripheral.connect().await {
                Ok(_) => {
                    println!("Connected!");
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // Essayons de découvrir les services immédiatement après la connexion
                    println!("Discovering services...");
                    match peripheral.discover_services().await {
                        Ok(_) => {
                            println!("Services discovered!");
                            tokio::time::sleep(Duration::from_secs(1)).await;

                            let chars = peripheral.characteristics();
                            println!("Found {} characteristics", chars.len());

                            if let Some(write_char) = chars.into_iter().find(|c| {
                                let uuid = c.uuid.to_string().to_uppercase();
                                uuid.contains("FFE9") || uuid.contains("FFF3")
                            }) {
                                println!("Found characteristic: {}", write_char.uuid);
                                println!("Ready to send commands");
                                return Ok(BleLedDevice {
                                    peripheral,
                                    write_characteristic: write_char,
                                });
                            }
                        },
                        Err(e) => println!("Service discovery failed: {}", e),
                    }
                },
                Err(e) => {
                    println!("Connection attempt failed: {}", e);
                    if attempt < 3 {
                        println!("Waiting before next attempt...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    } else {
                        return Err("Failed to connect after 3 attempts".to_string());
                    }
                }
            }
        }

        Err("Failed to establish connection and discover services".to_string())
    }

    // Helper function to write commands to the device
    pub async fn write_command(&self, command: &[u8]) -> Result<(), String> {
        self.peripheral.write(
            &self.write_characteristic,
            command,
            WriteType::WithoutResponse,
        ).await.map_err(|e| format!("Write failed: {}", e))
    }

    // Turn the device on or off
    pub async fn set_power(&self, on: bool) -> Result<(), String> {
        println!("Setting power: {}", if on { "ON" } else { "OFF" });
        let command = if on {
            [0x7e, 0x00, 0x04, 0xf0, 0x00, 0x01, 0xff, 0x00, 0xef]
        } else {
            [0x7e, 0x00, 0x04, 0x00, 0x00, 0x00, 0xff, 0x00, 0xef]
        };
        self.write_command(&command).await
    }

    // Set RGB color values
    pub async fn set_color(&self, r: u8, g: u8, b: u8) -> Result<(), String> {
        println!("Setting color to RGB({}, {}, {})", r, g, b);
        let command = [0x7e, 0x00, 0x05, 0x03, r, g, b, 0x00, 0xef];
        self.write_command(&command).await
    }

    // Set brightness level (0-100%)
    pub async fn set_brightness(&self, brightness: u8) -> Result<(), String> {
        let value = brightness.min(0x64); // Max 100%
        println!("Setting brightness to {}", value);
        let command = [0x7e, 0x00, 0x01, value, 0x00, 0x00, 0x00, 0x00, 0xef];
        self.write_command(&command).await
    }

    // Set warm white mode with brightness
    pub async fn set_warm_white(&self, brightness: u8) -> Result<(), String> {
        println!("Setting warm white to {}", brightness);
        let command = [0x7e, 0x00, 0x05, 0x02, brightness, brightness, brightness, 0x00, 0xef];
        self.write_command(&command).await
    }

    // Set custom animation effect with speed
    pub async fn set_custom_effect(&self, effect_code: u8, speed: u8) -> Result<(), String> {
        println!("Setting custom effect: {:#04x} with speed: {}", effect_code, speed);
        // First set the effect type
        let command1 = [0x7e, 0x00, 0x03, effect_code, 0x03, 0x00, 0x00, 0x00, 0xef];
        self.write_command(&command1).await?;
        
        // Then set the animation speed
        let speed_value = speed.min(0x64); // Max 100%
        let command2 = [0x7e, 0x00, 0x02, speed_value, 0x00, 0x00, 0x00, 0x00, 0xef];
        self.write_command(&command2).await
    }
}
