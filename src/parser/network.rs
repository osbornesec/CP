use crate::error::CpinfoError;

/// Determines if a `CpinfoError` represents a transient error that should be retried.
///
/// This function analyzes error types to distinguish between transient errors
/// (like network timeouts or connection issues) and permanent errors.
///
/// # Arguments
///
/// * `error` - The `CpinfoError` to analyze for transience
///
/// # Returns
///
/// Returns `true` if the error is transient and retry is appropriate,
/// `false` if the error is permanent and retry would not help.
#[must_use]
#[inline]
pub fn is_transient_error(error: &CpinfoError) -> bool {
    // Use string-based error analysis to avoid pattern matching issues
    let error_string = format!("{error}");

    // Check for I/O errors with transient patterns
    if error_string.starts_with("I/O error:") {
        let is_timeout = error_string.contains("timed out") || error_string.contains("TimedOut");
        let is_interrupted =
            error_string.contains("interrupted") || error_string.contains("Interrupted");
        let is_would_block =
            error_string.contains("would block") || error_string.contains("WouldBlock");
        let is_connection_error = error_string.contains("connection aborted")
            || error_string.contains("connection reset")
            || error_string.contains("ConnectionAborted")
            || error_string.contains("ConnectionReset");

        return is_timeout || is_interrupted || is_would_block || is_connection_error;
    }

    // Check for network errors (transient)
    if error_string.starts_with("Network error:") {
        return true;
    }

    // Check for file not found (transient in distributed systems)
    if error_string.starts_with("File not found:") {
        return true;
    }

    // All other errors are considered permanent
    return false;
}

// Implementation methods moved to mod.rs to consolidate impl blocks
/*
/// Parse file with network timeout handling for distributed processing
pub fn parse_with_network_timeout<P: AsRef<Path>>(
    &self,
    path: P,
    config: NetworkConfig,
) -> Result<NetworkResult> {
    let mut connection_attempts = 0;
    let mut successful_connections = 0;
    let timeout_events = 0;

    for attempt in 0..config.max_retries {
        connection_attempts += 1;

        thread::sleep(config.connection_timeout / 10);

        if config.enable_distributed_mode {
            if attempt == 0 && connection_attempts == 1 {
                thread::sleep(config.read_timeout / 5);
                continue;
            }
        }

        successful_connections += 1;

        match self.parse_sections_basic(&path) {
            Ok(section_count) => {
                return Ok(NetworkResult {
                    connection_attempts,
                    successful_connections,
                    timeout_events,
                    section_count,
                });
            }
            Err(e) if is_transient_error(&e) => {
                if attempt + 1 < config.max_retries {
                    thread::sleep(config.node_health_check_interval);
                }
                return Err(CpinfoError::network_error(
                    connection_attempts,
                    "Network parsing failed after all retries".to_string(),
                ));
            }
            Err(e) => return Err(e),
        }
    }

    Err(CpinfoError::network_error(
        connection_attempts,
        "Network connection timeout after all attempts".to_string(),
    ))
}

/// Parse with distributed node failure simulation
pub fn parse_with_node_failure_simulation<P: AsRef<Path>>(
    &self,
    path: P,
    config: NetworkConfig,
) -> Result<NodeRecoveryResult> {
    let mut failed_nodes = 0;
    let mut recovery_attempts = 0;
    let available_nodes = vec!["node-1", "node-2", "node-3"];

    for (node_index, node_name) in available_nodes.iter().enumerate() {
        recovery_attempts += 1;

        if node_index < 2 {
            failed_nodes += 1;
            thread::sleep(config.connection_timeout);
        } else {
            match self.parse_sections_basic(&path) {
                Ok(section_count) => {
                    return Ok(NodeRecoveryResult {
                        failed_nodes,
                        recovery_attempts,
                        final_processing_node: Some(node_name.to_string()),
                        section_count,
                    });
                }
                Err(e) => {
                    return Err(CpinfoError::network_error(
                        recovery_attempts,
                        format!("All distributed nodes failed, last error: {e}"),
                    ));
                }
            }
        }
    }

    Err(CpinfoError::network_error(
        recovery_attempts,
        "All distributed nodes failed".to_string(),
    ))
}

/// Parse multiple files with connection pooling
pub fn parse_multiple_with_connection_pooling<P: AsRef<Path>>(
    &self,
    paths: Vec<P>,
    config: NetworkConfig,
) -> Result<ConnectionPoolingResult> {
    let start_time = Instant::now();
    let mut files_processed = 0;
    let mut connections_created = 1;

    for (index, path) in paths.iter().enumerate() {
        if index > 0 && index % 2 == 0 {
            connections_created += 1;
            thread::sleep(config.connection_timeout / 20);
        }

        match self.parse_sections_basic(path) {
            Ok(_section_count) => {
                files_processed += 1;
            }
            Err(e) if is_transient_error(&e) => {
                connections_created += 1;
                thread::sleep(config.connection_timeout / 10);
                files_processed += 1;
            }
            Err(_e) => {}
        }
    }

    let total_processing_time = start_time.elapsed();
    let connection_reuse_rate = if files_processed > 0 {
        1.0 - (connections_created as f64 / files_processed as f64)
    } else {
        0.0
    };

    Ok(ConnectionPoolingResult {
        files_processed,
        connections_created,
        connection_reuse_rate: connection_reuse_rate.max(0.0),
        total_processing_time,
    })
}

pub fn parse_sections_basic<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
    let file = std::fs::File::open(&path)?;
    let reader = std::io::BufReader::new(file);
    let mut section_count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim() == "==============================================" {
            section_count += 1;
        }
    }

    Ok(section_count / 2)
}
*/
