use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MewtionConfig {
    pub dot_size: f64,
    pub color_rgb: (f64, f64, f64),
    pub opacity: f64,
    pub margin_pct: f64,
    pub sensitivity: f64,
    pub animation_mode: String,
}

impl Default for MewtionConfig {
    fn default() -> Self {
        Self {
            dot_size: 16.0,
            color_rgb: (1.0, 1.0, 1.0), // Default White
            opacity: 0.85,              // 85% visible
            margin_pct: 0.03,           // 3% from the edge
            sensitivity: 3.5,           // Default acceleration division scalar
            animation_mode: "Fluid".to_string(),
        }
    }
}

impl MewtionConfig {
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
                    "size" => if let Ok(v) = value.parse::<f64>() { config.dot_size = v; }
                    "color" => if let Some(rgb) = parse_hex_color(value) { config.color_rgb = rgb; }
                    "opacity" => if let Ok(v) = value.parse::<f64>() { config.opacity = v; }
                    "margin" => if let Ok(v) = value.parse::<f64>() { config.margin_pct = v; }
                    "sensitivity" => if let Ok(v) = value.parse::<f64>() { config.sensitivity = v; }
                    "animation" => config.animation_mode = value.to_string(),
                    _ => {}
                }
            }
        }
        config
    }
}

fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64)> {
    let clean_hex = hex.trim().trim_start_matches('#');
    if clean_hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&clean_hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean_hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean_hex[4..6], 16).ok()?;
    Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
}
