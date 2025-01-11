use clap::{Arg, Command};
use std::{
    fs::File,
    io::{BufReader, BufRead},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
    },
    thread,
    time::Duration,
};
use csv::Writer;
use chrono::Local;

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

    // Initialize CSV writer and protect it with Mutex for thread-safe access
    let csv_file = File::create("output.csv")
        .unwrap_or_else(|e| panic!("Failed to create CSV file: {}", e));
    let writer = Writer::from_writer(csv_file);
    let writer = Arc::new(Mutex::new(writer));

    // Write CSV headers
    {
        let mut w = writer.lock().unwrap();
        let headers = vec!["Type", "Timestamp", "Run/End", "Value1", "Value2", "Value3", "Value4"];
        w.write_record(&headers).expect("Failed to write CSV headers");
        w.flush().expect("Failed to flush CSV writer");
    }

    // Shared atomic flag to control recording
    let recording = Arc::new(AtomicBool::new(false));

    // Clone for serial thread
    let recording_clone = Arc::clone(&recording);
    let writer_clone = Arc::clone(&writer);
    let port_name_for_thread = port_name.clone();

    // Spawn serial thread to handle incoming serial data
    let serial_thread = thread::spawn(move || {
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

                    // Process only lines containing "UDP packet contents:"
                    if data.contains("UDP packet contents:") {
                        if recording_clone.load(Ordering::Acquire) {
                            let timestamp = get_timestamp();

                            // Extract the actual UDP contents after the colon
                            if let Some((_, payload)) = data.split_once(':') {
                                let payload = payload.trim(); // e.g., "7551870,-2.45,-3.69,-9.15"

                                // Split the payload by commas
                                let fields: Vec<&str> = payload.split(',').collect();

                                // Ensure the payload has the expected number of fields (4)
                                let expected_len = 4;
                                if fields.len() == expected_len {
                                    let record = vec![
                                        "data",
                                        &timestamp,
                                        "",
                                        fields[0],
                                        fields[1],
                                        fields[2],
                                        fields[3],
                                    ];

                                    // Write the record to CSV
                                    let mut w = writer_clone.lock().unwrap();
                                    if let Err(e) = w.write_record(&record) {
                                        eprintln!("Failed to write data record to CSV: {}", e);
                                    }
                                    if let Err(e) = w.flush() {
                                        eprintln!("Failed to flush CSV writer: {}", e);
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

    // Main thread: handle user commands
    let mut run_num :i64 = 0;
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
                if !recording.load(Ordering::Relaxed) {
                    recording.store(true, Ordering::Relaxed);
                    println!("Recording started.");

                    // Write start marker to CSV
                    let timestamp = get_timestamp();
                    let run_str = format!("run {}", run_num); // You can implement run numbering if needed
                    run_num += 1;
                    let start_record = vec!["start", &timestamp, &run_str, "", "", "", ""];
                    let mut w = writer.lock().unwrap();
                    if let Err(e) = w.write_record(&start_record) {
                        eprintln!("Failed to write start record to CSV: {}", e);
                    }
                    if let Err(e) = w.flush() {
                        eprintln!("Failed to flush CSV writer: {}", e);
                    }
                } else {
                    println!("Recording is already started.");
                }
            }
            "stop" => {
                if recording.load(Ordering::Relaxed) {
                    recording.store(false, Ordering::Relaxed);
                    println!("Recording stopped.");

                    // Write stop marker to CSV
                    let timestamp = get_timestamp();
                    let stop_record = vec!["stop", &timestamp, "end of run", "", "", "", ""];
                    let mut w = writer.lock().unwrap();
                    if let Err(e) = w.write_record(&stop_record) {
                        eprintln!("Failed to write stop record to CSV: {}", e);
                    }
                    if let Err(e) = w.flush() {
                        eprintln!("Failed to flush CSV writer: {}", e);
                    }
                } else {
                    println!("Recording is not active.");
                }
            }
            "exit" => {
                println!("Exiting...");

                // If recording is active, stop it first
                if recording.load(Ordering::Relaxed) {
                    recording.store(false, Ordering::Relaxed);
                    println!("Recording stopped.");

                    // Write stop marker to CSV
                    let timestamp = get_timestamp();
                    let stop_record = vec!["stop", &timestamp, "end of run", "", "", "", ""];
                    let mut w = writer.lock().unwrap();
                    if let Err(e) = w.write_record(&stop_record) {
                        eprintln!("Failed to write stop record to CSV: {}", e);
                    }
                    if let Err(e) = w.flush() {
                        eprintln!("Failed to flush CSV writer: {}", e);
                    }
                }

                // Terminate the program
                // Note: This will forcibly terminate the serial thread
                std::process::exit(0);
            }
            _ => {
                println!("Unknown command. Use 'start', 'stop', or 'exit'.");
            }
        }
    }
}

// Function to get the current timestamp in "YYYY-MM-DD HH:MM:SS" format
fn get_timestamp() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}
