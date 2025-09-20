use std::fs::File;
use std::io::Write;
use std::process::Command;
use log::{info, error};
use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, ColorChoice, TermLogger, WriteLogger, CombinedLogger, TerminalMode};
use warp::Filter;

#[derive(Deserialize)]
struct SendRequest {
    slatepack: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let log_file = File::create("finalize.log").unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
    ]).unwrap();

    // Define the POST route
    let send_finalize = warp::post()
        .and(warp::path("finalize"))
        .and(warp::body::json())
        .map(|request: SendRequest| {
            // Extract the slatepack message from the request
            let slatepack = request.slatepack;

            // Write slatepack to a file
            let file_path = "/home/grin/grin-finalize/slatepack.tmp";
            if let Err(e) = write_to_file(file_path, &slatepack) {
                error!("Failed to write slatepack to file: {:?}", e);
                return warp::reply::json(&Response {
                    message: "Failed to write slatepack to file".to_string(),
                });
            }

            // Execute the command
            let output = Command::new("bash")
                .arg("-c")
                .arg("python3 /home/grin/grin-finalize/finalize.py")
                .output()
                .expect("Failed to execute command");

            // Process the command output
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _exit_status = output.status;

            // Log the outputs for debugging
            info!("Command stdout: {}", stdout);
            error!("Command stderr: {}", stderr);

            // Check for specific success messages in stdout
            if stdout.contains("The slatepack data is NOT encrypted") && stdout.contains("Command 'finalize' completed successfully") {
                info!("Transaction finalized: {}", stdout);
                return warp::reply::json(&Response {
                    message: "Grin transaction successfully finalized ツ".to_string(),
                });
            } else {
                let error_message = if !stderr.is_empty() {
                    stderr.to_string()
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    "Command executed but produced no output.".to_string()
                };

                error!("Failed to finalize transaction: {}", error_message);
                return warp::reply::json(&Response {
                    message: format!("Error: {}", error_message),
                });
            }
        });

    // Load SSL keys and certs
    let cert_path = "/etc/ssl/cert.pem";
    let key_path = "/etc/ssl/privkey.pem";

    // Enable CORS
    let cors = warp::cors()
        .allow_origin("https://faucet.grinminer.net")
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["Content-Type"]);

    // Start the Warp server with CORS and TLS
    warp::serve(send_finalize.with(cors))
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .run(([0, 0, 0, 0], 3033)) // Listen on all interfaces
        .await;
}

// Function to write slatepack to a file
fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}