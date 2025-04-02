# Serial Logger
Reads serial data and stores it in a CSV. 

## Getting Started

1. Install Rustup for your machine following the[ link.](https://rustup.rs/)
2. 


## USAGE:
**Generic**
```bash
serial_logger --port <PORT> [--baud <BAUD>] [--output <OUTPUT>]
```
**Windows**
```bash
serial_logger.exe --port COM3 --baud 9600 --output C:\Users\username\Documents\sensor_data.csv
```
**Linux**
```bash
    ./serial_logger --port /dev/ttyACM0 --baud 9600 --output /home/username/data/sensor_data.csv
```

**List of Options**
```
OPTIONS:
  -p, --port <PORT>      Serial port to connect to (e.g., COM3 or /dev/ttyUSB0)
  -b, --baud <BAUD>      Baud rate for the serial port [default: 115200]
  -o, --output <OUTPUT>  Path to output CSV file [default: output.csv]
  -h, --help             Print help information
  -V, --version          Print version information
```

