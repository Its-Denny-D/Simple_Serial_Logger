use clap::{Arg, Command};
use std::{
    fs::File,
    io::{BufReader, BufRead},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    sync::mpsc::{channel, Sender, Receiver},
    thread,
    time::Duration,
};
use serialport::SerialPort; // Ensure you have the serialport crate in Cargo.toml
use csv::Writer;

fn main() {
    // Parse command-line arguments using Clap
    let matches = Command::new("Serial Logger")
        .version("1.0")
        .about("Reads serial data and stores it in a CSV")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Serial port to connect to (e.g., COM3 or /dev/ttyUSB0)")
                .required(true),
        )
        .arg(
            Arg::new("baud")
                .short('b')
                .long("baud")
                .value_name("BAUD")
                .help("Baud rate for the serial port (e.g., 115200)")
                .default_value("115200"),
        )
        .get_matches();

    // Retrieve command-line arguments
    let port_name = matches.get_one::<String>("port").expect("Port is required");
    let baud_rate: u32 = matches
        .get_one::<String>("baud")
        .expect("Baud rate has a default value")
        .parse()
        .expect("Failed to parse baud rate");

    // Shared atomic flag to control recording
    let recording = Arc::new(AtomicBool::new(false));
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    // Clone the recording flag and port name for the thread
    let recording_clone = Arc::clone(&recording);
    let port_name_for_thread = port_name.clone();

    // Spawn a thread to handle serial input
    thread::spawn(move || {
        // Open the serial port
        let port = serialport::new(&port_name_for_thread, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .unwrap_or_else(|e| panic!("Failed to open serial port {}: {}", port_name_for_thread, e));

        let mut reader = BufReader::new(port);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            // Read a line from the serial port
            match reader.read_line(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        // No data read; continue
                        continue;
                    }

                    // Clean the data by removing tab characters and trimming whitespace
                    let data = buffer.trim().replace('\t', "").to_string();

                    // Debug: Uncomment to see all received data
                    // println!("Received: {}", data);

                    // Process only lines containing "UDP packet contents:"
                    if data.contains("UDP packet contents:") {
                        // Send the cleaned data to the main thread if recording is active
                        if recording_clone.load(Ordering::Acquire) {
                            if let Err(e) = tx.send(data) {
                                eprintln!("Failed to send data to main thread: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from serial port: {}", e);
                }
            }

            // Sleep briefly to prevent high CPU usage
            thread::sleep(Duration::from_millis(10));
        }
    });

    // CSV writer setup
    let csv_file = File::create("output.csv").unwrap_or_else(|e| panic!("Failed to create CSV file: {}", e));
    let mut writer = Writer::from_writer(csv_file);

    // Optional: Write CSV headers
    // Uncomment the following lines if you want headers in your CSV
    /*
    if let Err(e) = writer.write_record(&["Timestamp", "Value1", "Value2", "Value3"]) {
        eprintln!("Failed to write CSV headers: {}", e);
    }
    writer.flush().unwrap();
    */

    // Command loop in the main thread
    loop {
        println!("Enter a command (start, stop, exit):");
        let mut command = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut command) {
            eprintln!("Failed to read input: {}", e);
            continue;
        }
        let command = command.trim();

        match command {
            "start" => {
                recording.store(true, Ordering::Relaxed);
                println!("Recording started.");
            }
            "stop" => {
                recording.store(false, Ordering::Relaxed);
                println!("Recording stopped.");
            }
            "exit" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Unknown command. Use 'start', 'stop', or 'exit'.");
            }
        }

        // Write data to CSV if available
        while let Ok(data) = rx.try_recv() {
            // Example data: "UDP packet contents: 185611,-2.85,-5.12,-8.35"

            // Extract the actual UDP contents after the colon
            if let Some((_, payload)) = data.split_once(':') {
                let payload = payload.trim(); // "185611,-2.85,-5.12,-8.35"

                // Split the payload by commas
                let fields: Vec<&str> = payload.split(',').collect();

                // Ensure the payload has the expected number of fields (e.g., 4)
                let expected_len = 4;
                if fields.len() == expected_len {
                    if let Err(e) = writer.write_record(&fields) {
                        eprintln!("Failed to write record to CSV: {}", e);
                    }
                } else {
                    eprintln!(
                        "Warning: Unexpected number of fields (expected {}, got {}). Data: {}",
                        expected_len,
                        fields.len(),
                        payload
                    );
                }
            } else {
                eprintln!("Warning: 'UDP packet contents:' not found in data: {}", data);
            }

            // Flush the writer to ensure data is written to the file
            if let Err(e) = writer.flush() {
                eprintln!("Failed to flush CSV writer: {}", e);
            }
        }
    }
}
