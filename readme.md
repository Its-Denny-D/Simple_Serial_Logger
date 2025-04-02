# Serial Logger
Reads serial data and stores it in a CSV. When running the user can select start to start data collection to the file. Stop pauses the data collection and exit ends the program.

## Getting Started

If you want to use the program without changes just download a release binary from the releases tab.

To pull down the code and modify it: 

1. Install Rustup for your machine following the[ link.](https://rustup.rs/)
2. Setup your preferred programming environment. 
3. Install git on [windows](https://git-scm.com/downloads/win) or use [linux](https://git-scm.com/downloads/linux) in the terminal with apt-get install git 
4. Git clone [repo-url] [example](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository)
5. Building with cargo is as simple as `cargo build` 


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

