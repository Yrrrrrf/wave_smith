#![allow(unused)]

use std::process::{Command, Child};
use std::time::Duration;
use std::thread;
use std::path::PathBuf;
use std::io::{self, Write};

use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};

struct TestConfig {
    threshold: f32,
    message: String,
    duration: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            threshold: 0.01,
            message: String::from("TEST MESSAGE"),
            duration: Duration::from_secs(5),
        }
    }
}

struct LocalTest {
    config: TestConfig,
    receiver_process: Option<Child>,
    sender_process: Option<Child>,
}

impl LocalTest {
    fn new(config: TestConfig) -> Self {
        Self {
            config,
            receiver_process: None,
            sender_process: None,
        }
    }

    fn start_process(&mut self, script_type: &str) -> io::Result<()> {
        info!("{}", format!("Starting {}...", script_type).color(BLUE).style(Style::Bold));
        
        let mut cmd = Command::new("cargo");
        
        // Prepare common args
        let mut args = vec!["run", "--example", script_type];
        
        // Add specific argument based on script type
        let specific_arg = match script_type {
            "receiver" => self.config.threshold.to_string(),
            "sender" => self.config.message.clone(),
            _ => return Err(io::Error::new(
                io::ErrorKind::InvalidInput, 
                format!("Invalid script type: {}", script_type)
            )),
        };
        args.push(&specific_arg);
        
        cmd.args(&args);
        debug!("{} command: {:?}", script_type, cmd);
        
        // Store process in appropriate field
        match script_type {
            "receiver" => {
                self.receiver_process = Some(cmd.spawn()?);
                // Give receiver time to initialize
                thread::sleep(Duration::from_millis(500));
            },
            "sender" => {
                self.sender_process = Some(cmd.spawn()?);
            },
            _ => unreachable!(),
        }
        
        info!("{}", format!("{} started successfully", script_type).color(GREEN));
        Ok(())
    }


    fn cleanup(&mut self) {
        info!("{}", "Cleaning up processes...".color(YELLOW));
        
        if let Some(mut receiver) = self.receiver_process.take() {
            match receiver.kill() {
                Ok(_) => debug!("Receiver process terminated"),
                Err(e) => error!("Failed to terminate receiver: {}", e),
            }
        }

        if let Some(mut sender) = self.sender_process.take() {
            match sender.kill() {
                Ok(_) => debug!("Sender process terminated"),
                Err(e) => error!("Failed to terminate sender: {}", e),
            }
        }
    }

    fn run_test(&mut self) -> io::Result<()> {
        println!("\n{}", "╔═══════════════════════════════════════════════".color(CYAN));
        println!("{} {}", 
            "║".color(CYAN),
            "LOCAL AUDIO TRANSMISSION TEST".color(WHITE).style(Style::Bold)
        );
        println!("{}", "╠═══════════════════════════════════════════════".color(CYAN));
        println!("{} {} {}", 
            "║".color(CYAN),
            "Message:".color(WHITE).style(Style::Bold),
            self.config.message.color(GREEN)
        );
        println!("{} {} {}", 
            "║".color(CYAN),
            "Threshold:".color(WHITE).style(Style::Bold),
            self.config.threshold.to_string().color(GREEN)
        );
        println!("{} {} {}", 
            "║".color(CYAN),
            "Duration:".color(WHITE).style(Style::Bold),
            format!("{:?}", self.config.duration).color(GREEN)
        );
        println!("{}\n", "╚═══════════════════════════════════════════════".color(CYAN));

        // * main test logic
        self.start_process("receiver")?;
        thread::sleep(Duration::from_millis(1000));
        self.start_process("sender")?;

        Ok(())
    }
}

impl Drop for LocalTest {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn main() -> io::Result<()> {
    app_dt!(file!());
    set_max_level(Level::Debug);

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let mut config = TestConfig::default();
    
    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--threshold" => {
                if i + 1 < args.len() {
                    config.threshold = args[i + 1].parse().unwrap_or(0.01);
                    i += 2;
                }
            }
            "-m" | "--message" => {
                if i + 1 < args.len() {
                    config.message = args[i + 1].clone();
                    i += 2;
                }
            }
            "-d" | "--duration" => {
                if i + 1 < args.len() {
                    config.duration = Duration::from_secs(args[i + 1].parse().unwrap_or(5));
                    i += 2;
                }
            }
            _ => i += 1,
        }
    }

    info!("Starting local transmission test...");
    
    let mut test = LocalTest::new(config);
    test.run_test()?;

    info!("{}", "Test completed successfully!".color(GREEN).style(Style::Bold));

    Ok(())
}