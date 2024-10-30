mod device;
use device::BleLedDevice;
use std::io::{self, Write};

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

impl Color {
    fn get_rgb(&self) -> (u8, u8, u8) {
        match self {
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

#[derive(Debug)]
enum Animation {
    Flash,
    Strobe,
    Fade,
    Smooth,
}

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

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    input.trim().parse().ok()
}

async fn show_warm_white_menu() -> Option<u8> {
    println!("\n=== Warm White Intensity ===");
    println!("1. High (100%)");
    println!("2. Medium (50%)");
    println!("3. Low (25%)");
    print!("\nChoose intensity: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    match input.trim().parse::<u32>().ok()? {
        1 => Some(255),
        2 => Some(128),
        3 => Some(64),
        _ => None,
    }
}

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

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
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

async fn show_brightness_menu() -> Option<u8> {
    println!("\n=== Brightness Control ===");
    println!("1. Increase Brightness");
    println!("2. Decrease Brightness");
    print!("\nChoose an option: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    Some(match input.trim().parse::<u32>().ok()? {
        1 => 255,
        2 => 128,
        _ => return None,
    })
}

async fn show_animation_menu() -> Option<Animation> {
    println!("\n=== Animation Selection ===");
    println!("1. Flash");
    println!("2. Strobe");
    println!("3. Fade");
    println!("4. Smooth");
    print!("\nChoose an animation: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    
    match input.trim().parse::<u32>().ok()? {
        1 => Some(Animation::Flash),
        2 => Some(Animation::Strobe),
        3 => Some(Animation::Fade),
        4 => Some(Animation::Smooth),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("Starting Bluetooth LED controller...");
    
    let device_address = "BE:32:03:82:3C:B1";
    let device = BleLedDevice::new_with_address(device_address).await?;
    
    device.set_power(true).await?;

    loop {
        match show_menu().await {
            Some(1) => {
                if let Some(color) = show_color_menu().await {
                    let (r, g, b) = color.get_rgb();
                    device.set_color(r, g, b).await?;
                }
            },
            Some(2) => {
                if let Some(brightness) = show_brightness_menu().await {
                    device.set_brightness(brightness).await?;
                }
            },
            Some(3) => {
                if let Some(animation) = show_animation_menu().await {
                    match animation {
                        Animation::Flash => device.set_mode(0x25).await?,
                        Animation::Strobe => device.set_mode(0x26).await?,
                        Animation::Fade => device.set_mode(0x27).await?,
                        Animation::Smooth => device.set_mode(0x28).await?,
                    }
                }
            },
            Some(4) => {
                if let Some(intensity) = show_warm_white_menu().await {
                    device.set_warm_white(intensity).await?;
                }
            },
            Some(5) => {
                device.set_power(false).await?;
                println!("Device turned off");
            },
            Some(0) => {
                device.set_power(false).await?;
                println!("Goodbye!");
                break;
            },
            _ => println!("Invalid option, please try again"),
        }
    }

    Ok(())
}
