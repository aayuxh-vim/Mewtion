use crate::sensor::MotionSample;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub fn run_tcp_bridge_blocking<F>(ip_ref: Arc<Mutex<String>>, mut on_sample: F)
where
    F: FnMut(MotionSample),
{
    loop {
        // Safely lock and read the current IP target
        let target_address = {
            let ip = ip_ref.lock().unwrap();
            format!("{}:8765", *ip)
        };

        match TcpStream::connect(&target_address) {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

                while let Ok(bytes_read) = reader.read_line(&mut line) {
                    if bytes_read == 0 { break; } // Connection closed

                    let parts: Vec<&str> = line.trim().split(',').collect();
                    if parts.len() == 6 {
                        if let (Ok(ax), Ok(ay), Ok(az), Ok(gx), Ok(gy), Ok(gz)) = (
                            parts[0].parse::<f32>(), parts[1].parse::<f32>(), parts[2].parse::<f32>(),
                            parts[3].parse::<f32>(), parts[4].parse::<f32>(), parts[5].parse::<f32>(),
                        ) {
                            on_sample(MotionSample { ax, ay, az, gx, gy, gz });
                        }
                    }
                    line.clear();
                }
            }
            Err(_) => {
                // If connection fails, wait 1 second and loop (which will fetch the latest IP string)
                sleep(Duration::from_millis(1000));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn bridge_emits_complete_samples_and_ignores_malformed_lines() {
        let listener = TcpListener::bind("127.0.0.1:8765").expect("bind bridge test server");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept bridge connection");
            stream
                .write_all(
                    b"not,enough,fields\n1,2,3,4,5,invalid\n1,-2.5,3.25,-4,5.5,-6.75\n",
                )
                .expect("send bridge test data");
        });

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            run_tcp_bridge_blocking(
                Arc::new(Mutex::new("127.0.0.1".to_owned())),
                move |sample| sender.send(sample).expect("record parsed sample"),
            );
        });

        let sample = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("receive the one valid motion sample");
        assert_eq!(sample.ax, 1.0);
        assert_eq!(sample.ay, -2.5);
        assert_eq!(sample.az, 3.25);
        assert_eq!(sample.gx, -4.0);
        assert_eq!(sample.gy, 5.5);
        assert_eq!(sample.gz, -6.75);
        assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());

        server.join().expect("bridge test server should finish");
    }
}
