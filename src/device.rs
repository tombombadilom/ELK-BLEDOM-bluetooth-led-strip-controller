use btleplug::api::{Central, Manager, Peripheral, WriteType, ScanFilter, CharPropFlags, Characteristic};
use btleplug::platform::{Manager as PlatformManager, Peripheral as PlatformPeripheral};
use std::time::Duration;

pub struct BleLedDevice {
    peripheral: PlatformPeripheral,
    write_characteristic: Characteristic,
}

impl BleLedDevice {
    pub async fn new_with_address(addr_str: &str) -> Result<Self, String> {
        let manager = PlatformManager::new().await
            .map_err(|e| format!("Failed to create manager: {}", e))?;

        let adapters = manager.adapters().await
            .map_err(|e| format!("Failed to get adapters: {}", e))?;

        let central = adapters.into_iter().next()
            .ok_or("No Bluetooth adapters found")?;

        let _ = central.stop_scan().await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Starting scan...");
        central.start_scan(ScanFilter::default()).await
            .map_err(|e| format!("Failed to start scan: {}", e))?;

        tokio::time::sleep(Duration::from_secs(3)).await;

        let peripherals = central.peripherals().await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        let address = btleplug::api::BDAddr::from_str_delim(addr_str)
            .map_err(|e| format!("Invalid address format: {}", e))?;

        let peripheral = peripherals.into_iter()
            .find(|p| p.address() == address)
            .ok_or("Device not found")?;

        println!("Found device, stopping scan...");
        let _ = central.stop_scan().await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        if peripheral.is_connected().await.unwrap_or(false) {
            println!("Device was connected, disconnecting first...");
            let _ = peripheral.disconnect().await;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        let mut retry_count = 0;
        let max_retries = 3;
        let mut characteristics = Vec::new();

        while retry_count < max_retries {
            println!("Connection attempt {} of {}", retry_count + 1, max_retries);

            tokio::time::sleep(Duration::from_secs(2)).await;

            match peripheral.connect().await {
                Ok(_) => {
                    println!("Connected, waiting before service discovery...");
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    if peripheral.is_connected().await.unwrap_or(false) {
                        match peripheral.discover_services().await {
                            Ok(_) => {
                                println!("Services discovered, getting characteristics...");
                                tokio::time::sleep(Duration::from_secs(1)).await;

                                characteristics = peripheral.characteristics().into_iter().collect();
                                println!("Found {} characteristics", characteristics.len());

                                for c in &characteristics {
                                    println!("Characteristic UUID: {:?}", c.uuid);
                                    println!("Properties: {:?}", c.properties);
                                }

                                if !characteristics.is_empty() {
                                    break;
                                }
                            }
                            Err(e) => println!("Service discovery error: {}", e),
                        }
                    } else {
                        println!("Lost connection before service discovery");
                    }
                }
                Err(e) => {
                    println!("Connection error: {}", e);
                    if e.to_string().contains("already in progress") {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }

            let _ = peripheral.disconnect().await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            retry_count += 1;
            if retry_count < max_retries {
                println!("Waiting before next attempt...");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }

        if characteristics.is_empty() {
            return Err("Failed to discover any characteristics".to_string());
        }

        // Trouver la caractéristique d'écriture une seule fois
        let write_characteristic = characteristics.into_iter()
            .find(|c| c.uuid.to_string().to_uppercase().contains("FFF3") &&
                 (c.properties.contains(CharPropFlags::WRITE) || 
                  c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)))
            .ok_or("No suitable write characteristic found")?;

        println!("Using characteristic: {:?}", write_characteristic.uuid);

        Ok(BleLedDevice {
            peripheral,
            write_characteristic,
        })
    }

    pub async fn write_command(&self, command: &[u8]) -> Result<(), String> {
        // Reconnecter si nécessaire
        if !self.peripheral.is_connected().await.unwrap_or(false) {
            println!("Reconnecting...");
            self.peripheral.connect().await
                .map_err(|e| format!("Failed to connect: {}", e))?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Écrire la commande
        self.peripheral.write(
            &self.write_characteristic,
            command,
            WriteType::WithoutResponse,
        ).await.map_err(|e| format!("Write failed: {}", e))?;

        Ok(())
    }

    pub async fn set_color(&self, r: u8, g: u8, b: u8) -> Result<(), String> {
        println!("Setting color to RGB({}, {}, {})", r, g, b);
        let command = [0x56, r, g, b, 0x00, 0xF0, 0xAA];
        self.write_command(&command).await
    }

    pub async fn set_power(&self, on: bool) -> Result<(), String> {
        println!("Setting power: {}", if on { "ON" } else { "OFF" });
        let command = if on { [0xCC, 0x23, 0x33] } else { [0xCC, 0x24, 0x33] };
        self.write_command(&command).await
    }

    pub async fn set_brightness(&self, brightness: u8) -> Result<(), String> {
        println!("Setting brightness to {}", brightness);
        let command = [0x56, brightness, brightness, brightness, 0x00, 0xF0, 0xAA];
        self.write_command(&command).await
    }

    pub async fn set_warm_white(&self, brightness: u8) -> Result<(), String> {
        println!("Setting warm white to {}", brightness);
        let command = [0x56, brightness, brightness, 0x00, 0x0F, 0xAA];
        self.write_command(&command).await
    }

    pub async fn set_mode(&self, mode: u8) -> Result<(), String> {
        println!("Setting mode to: {:#04x}", mode);
        let command = [0xBB, mode, 0x44];
        self.write_command(&command).await
    }
}
