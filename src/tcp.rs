use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct MotionSample {
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub gx: f32,
    pub gy: f32,
    pub gz: f32,
}

pub fn run_tcp_bridge_blocking<F>(mut on_sample: F)
where
    F: FnMut(MotionSample),
{
    loop {
        match TcpStream::connect("127.0.0.1:8765") {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

                while let Ok(bytes_read) = reader.read_line(&mut line) {
                    if bytes_read == 0 {
                        break; // Connection closed
                    }

                    let parts: Vec<&str> = line.trim().split(',').collect();
                    if parts.len() == 6 {
                        if let (Ok(ax), Ok(ay), Ok(az), Ok(gx), Ok(gy), Ok(gz)) = (
                            parts[0].parse::<f32>(),
                            parts[1].parse::<f32>(),
                            parts[2].parse::<f32>(),
                            parts[3].parse::<f32>(),
                            parts[4].parse::<f32>(),
                            parts[5].parse::<f32>(),
                        ) {
                            on_sample(MotionSample { ax, ay, az, gx, gy, gz });
                        }
                    }
                    line.clear();
                }
            }
            Err(_) => {
                // Wait before retrying connection
                sleep(Duration::from_millis(1000));
            }
        }
    }
}
