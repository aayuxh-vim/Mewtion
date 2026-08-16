use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MewtionConfig {
    pub dot_size: f64,
    /// RGB color components normalized from 0.0 to 1.0 for Cairo/GTK rendering
    pub color_rgb: (f64, f64, f64),
}

impl Default for MewtionConfig {
    fn default() -> Self {
        Self {
            dot_size: 16.0,
            // Default Gruvbox Green (#98971a)
            color_rgb: (0.596, 0.592, 0.102),
        }
    }
}

impl MewtionConfig {
    /// Loads configuration from `mewtion_config.txt`.
    /// Falls back to default values if the file is missing or contains invalid syntax.
    pub fn load() -> Self {
        let path = "mewtion_config.txt";
        if !Path::new(path).exists() {
            return Self::default();
        }

        let mut config = Self::default();

        if let Ok(contents) = fs::read_to_string(path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() != 2 {
                    continue;
                }

                let key = parts[0].trim();
                let value = parts[1].trim();

                match key {
                    "size" => {
                        if let Ok(parsed_size) = value.parse::<f64>() {
                            config.dot_size = parsed_size;
                        }
                    }
                    "color" => {
                        if let Some(rgb) = parse_hex_color(value) {
                            config.color_rgb = rgb;
                        }
                    }
                    _ => {}
                }
            }
        }

        config
    }
}

/// Helper function to parse hex color strings like "#98971a" or "98971a" into (r, g, b) floats (0.0 .. 1.0)
fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64)> {
    let clean_hex = hex.trim().trim_start_matches('#');
    if clean_hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&clean_hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean_hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean_hex[4..6], 16).ok()?;

    Some((
        r as f64 / 255.0,
        g as f64 / 255.0,
        b as f64 / 255.0,
    ))
}
