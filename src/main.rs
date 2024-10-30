// Import required modules and dependencies
mod device;
use device::BleLedDevice;
use std::io::{self, Write};
use tokio::time::Duration;

// Enum representing available colors with RGB values
#[derive(Debug)]
enum Color {
    White,
    Blue,
    Purple,
    Pink,
    Turquoise,
    Green,
    Yellow,
    Orange,
    Red,
}

// Implementation block for Color enum to get RGB values
impl Color {
    fn get_rgb(&self) -> (u8, u8, u8) {
        match self {
            // Return RGB tuples for each color
            Color::White => (255, 255, 255),
            Color::Blue => (0, 0, 255),
            Color::Purple => (128, 0, 128),
            Color::Pink => (255, 192, 203),
            Color::Turquoise => (64, 224, 208),
            Color::Green => (0, 255, 0),
            Color::Yellow => (255, 255, 0),
            Color::Orange => (255, 165, 0),
            Color::Red => (255, 0, 0),
        }
    }
}

// Enum representing available animations
#[derive(Debug)]
enum Animation {
    JumpRGB,
    JumpRGBYCMW,
    CrossfadeRGB,
    CrossfadeRGBYCMW,
    BlinkRGBYCMW,
}

// Implementation block for Animation enum to get animation codes
impl Animation {
    fn get_code(&self) -> u8 {
        match self {
            // Return hex codes for each animation type
            Animation::JumpRGB => 0x87,
            Animation::JumpRGBYCMW => 0x88,
            Animation::CrossfadeRGB => 0x89,
            Animation::CrossfadeRGBYCMW => 0x8a,
            Animation::BlinkRGBYCMW => 0x95,
        }
    }
}

// Function to display main menu and get user input
async fn show_menu() -> Option<u32> {
    println!("\n=== LED Controller Menu ===");
    println!("1. Change Color");
    println!("2. Adjust Brightness");
    println!("3. Animations");
    println!("4. Warm White");
    println!("5. Turn Off");
    println!("0. Exit");
    print!("\nChoose an option: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    input.trim().parse().ok()
}

// Function to display color selection menu and get user choice
async fn show_color_menu() -> Option<Color> {
    println!("\n=== Color Selection ===");
    println!("1. White");
    println!("2. Blue");
    println!("3. Purple");
    println!("4. Pink");
    println!("5. Turquoise");
    println!("6. Green");
    println!("7. Yellow");
    println!("8. Orange");
    println!("9. Red");
    print!("\nChoose a color: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    // Match user input to corresponding color
    match input.trim().parse::<u32>().ok()? {
        1 => Some(Color::White),
        2 => Some(Color::Blue),
        3 => Some(Color::Purple),
        4 => Some(Color::Pink),
        5 => Some(Color::Turquoise),
        6 => Some(Color::Green),
        7 => Some(Color::Yellow),
        8 => Some(Color::Orange),
        9 => Some(Color::Red),
        _ => None,
    }
}

// Function to display brightness control menu and get user choice
async fn show_brightness_menu() -> Option<u8> {
    println!("\n=== Brightness Control ===");
    println!("1. 100%");
    println!("2. 75%");
    println!("3. 50%");
    println!("4. 25%");
    print!("\nChoose brightness level: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    // Match user input to brightness percentage
    match input.trim().parse::<u32>().ok()? {
        1 => Some(100),
        2 => Some(75),
        3 => Some(50),
        4 => Some(25),
        _ => None,
    }
}

// Function to display animation selection menu and get user choice
async fn show_animation_menu() -> Option<Animation> {
    println!("\n=== Animation Selection ===");
    println!("1. Jump RGB");
    println!("2. Jump All Colors");
    println!("3. Crossfade RGB");
    println!("4. Crossfade All Colors");
    println!("5. Blink All Colors");
    print!("\nChoose an animation: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    // Match user input to corresponding animation
    match input.trim().parse::<u32>().ok()? {
        1 => Some(Animation::JumpRGB),
        2 => Some(Animation::JumpRGBYCMW),
        3 => Some(Animation::CrossfadeRGB),
        4 => Some(Animation::CrossfadeRGBYCMW),
        5 => Some(Animation::BlinkRGBYCMW),
        _ => None,
    }
}

// Function to display animation speed menu and get user choice
async fn show_animation_speed_menu() -> Option<u8> {
    println!("\n=== Animation Speed ===");
    println!("1. Fast (100%)");
    println!("2. Medium (50%)");
    println!("3. Slow (25%)");
    print!("\nChoose speed: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    // Match user input to speed percentage
    match input.trim().parse::<u32>().ok()? {
        1 => Some(100),
        2 => Some(50),
        3 => Some(25),
        _ => None,
    }
}

// Function to display warm white intensity menu and get user choice
async fn show_warm_white_menu() -> Option<u8> {
    println!("\n=== Warm White Intensity ===");
    println!("1. High (100%)");
    println!("2. Medium (75%)");
    println!("3. Low (50%)");
    println!("4. Very Low (25%)");
    print!("\nChoose intensity: ");
    io::stdout().flush().unwrap();

    // Read and parse user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    // Match user input to intensity value
    match input.trim().parse::<u32>().ok()? {
        1 => Some(255),
        2 => Some(191),
        3 => Some(128),
        4 => Some(64),
        _ => None,
    }
}

// Main async function using tokio runtime
#[tokio::main]
async fn main() -> Result<(), String> {
    println!("Starting Bluetooth LED controller...");
    
    // Initialize device with hardcoded Bluetooth address
    let device_address = "BE:32:03:82:3C:B1";
    let device = BleLedDevice::new_with_address(device_address).await?;
    
    // Turn on device initially
    println!("Turning on...");
    device.set_power(true).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Main program loop
    loop {
        match show_menu().await {
            // Handle color change
            Some(1) => {
                if let Some(color) = show_color_menu().await {
                    let (r, g, b) = color.get_rgb();
                    println!("Setting color...");
                    device.set_color(r, g, b).await?;
                }
            },
            // Handle brightness adjustment
            Some(2) => {
                if let Some(brightness) = show_brightness_menu().await {
                    println!("Setting brightness...");
                    device.set_brightness(brightness).await?;
                }
            },
            // Handle animation selection
            Some(3) => {
                if let Some(animation) = show_animation_menu().await {
                    if let Some(speed) = show_animation_speed_menu().await {
                        println!("Setting animation with speed...");
                        device.set_custom_effect(animation.get_code(), speed).await?;
                    }
                }
            },
            // Handle warm white adjustment
            Some(4) => {
                if let Some(intensity) = show_warm_white_menu().await {
                    println!("Setting warm white...");
                    device.set_warm_white(intensity).await?;
                }
            },
            // Handle power off
            Some(5) => {
                println!("Turning off...");
                device.set_power(false).await?;
            },
            // Handle exit
            Some(0) => {
                println!("Turning off and exiting...");
                device.set_power(false).await?;
                break;
            },
            // Handle invalid input
            _ => println!("Invalid option, please try again"),
        }
        // Small delay between commands
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
