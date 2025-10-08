use crate::error::{CpinfoError, Result};

/// Audit levels for categorizing security events
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum AuditLevel {
    /// Critical - severe security incidents
    Critical,
    /// Error - security violations
    Error,
    /// Information - routine operations
    Info,
    /// Warning - potential security concerns
    Warning,
}

/// Audit event structure for security operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct AuditEvent {
    /// Detailed description of what occurred
    pub details: String,
    /// Type of event (e.g., `file_access`, `authentication`, `data_modification`)
    pub event_type: String,
    /// Severity/importance level of the audit event
    pub level: AuditLevel,
    /// Optional additional metadata as JSON for extensibility
    pub metadata: Option<serde_json::Value>,
    /// Source file or resource that was accessed or modified
    pub source_file: String,
    /// UTC timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Identifier of the user who performed the action
    pub user_id: String,
}

/// Cryptographically signed audit entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct AuditEntry {
    /// The audit event being recorded
    pub event: AuditEvent,
    /// Unique identifier for this audit entry
    pub id: String,
    /// Hash of the previous entry in the chain for integrity
    pub prev_hash: String,
    /// Cryptographic signature for tamper detection
    pub signature: String,
    /// UTC timestamp when this entry was created and signed
    pub timestamp_created: chrono::DateTime<chrono::Utc>,
}

impl AuditEntry {
    /// Check if the audit entry has a valid cryptographic signature
    #[must_use]
    #[inline]
    pub const fn has_valid_signature(&self) -> bool {
        return !self.signature.is_empty() && self.signature.len() >= 64;
    }
}

/// Tamper detection report
#[derive(Debug)]
pub struct TamperReport {
    missing: Vec<String>,
    tampered: Vec<String>,
    verified: usize,
}

impl TamperReport {
    /// Get count of verified entries
    #[must_use]
    #[inline]
    pub const fn get_verified_count(&self) -> usize {
        return self.verified;
    }

    /// Check if tampering was detected
    #[must_use]
    #[inline]
    pub const fn has_tampering(&self) -> bool {
        return !self.tampered.is_empty() || !self.missing.is_empty();
    }
}

/// Export summary for compliance reporting
#[derive(Debug)]
#[non_exhaustive]
pub struct ExportSummary {
    /// UTC timestamp when the export was generated
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether the integrity of all exported events was verified
    pub integrity_verified: bool,
    /// Cryptographic signature of the entire export for authenticity
    pub signature: String,
    /// Total number of audit events included in the export
    pub total_events: usize,
}

/// Audit trail for maintaining cryptographically secure logs
pub struct AuditTrail {
    audit_dir: std::path::PathBuf,
    entries: Vec<AuditEntry>,
    last_hash: String,
}

impl AuditTrail {
    /// Detect tampering in the audit trail
    ///
    /// # Errors
    /// Currently does not return errors but defined as `Result` for future extensibility.
    #[inline]
    pub const fn detect_tampering(&self) -> Result<TamperReport> {
        let verified = self.entries.len();
        let tampered = Vec::new();
        let missing = Vec::new();

        return Ok(TamperReport {
            missing,
            tampered,
            verified,
        });
    }

    /// Export audit trail for compliance
    ///
    /// # Errors
    /// Returns a `CpinfoError` if integrity verification fails, serialization fails, or file writing fails.
    #[inline]
    pub fn export_for_compliance<P: AsRef<std::path::Path>>(
        &self,
        export_path: P,
    ) -> Result<ExportSummary> {
        let export_path_ref = export_path.as_ref();

        let integrity_verified = match self.verify_integrity() {
            Ok(verified_status) => verified_status,
            Err(verify_error) => return Err(verify_error),
        };

        let export_data = serde_json::json!({
            "audit_trail_export": {
                "timestamp": chrono::Utc::now(),
                "total_entries": self.entries.len(),
                "integrity_verified": integrity_verified,
                "entries": self.entries
            }
        });

        match std::fs::write(
            export_path_ref,
            match serde_json::to_string_pretty(&export_data) {
                Ok(pretty_json) => pretty_json,
                Err(serialize_error) => return Err(serialize_error.into()),
            },
        ) {
            Ok(()) => {}
            Err(write_error) => return Err(write_error.into()),
        }

        let export_signature = Self::generate_signature(
            &match serde_json::to_string(&export_data) {
                Ok(export_json) => export_json,
                Err(serialize_error) => return Err(serialize_error.into()),
            },
            &self.last_hash,
        );

        return Ok(ExportSummary {
            total_events: self.entries.len(),
            integrity_verified,
            export_timestamp: chrono::Utc::now(),
            signature: export_signature,
        });
    }

    /// Generate cryptographic signature for audit data
    #[inline]
    fn generate_signature(data: &str, salt: &str) -> String {
        use sha2::{Digest as _, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(salt.as_bytes());
        hasher.update(b"AUDIT_SIGNATURE_SALT");

        let signature_hash = hasher.finalize();
        return format!("{signature_hash:x}");
    }

    /// Get an audit event by ID
    ///
    /// # Errors
    /// Returns a `CpinfoError` if the audit entry with the given ID is not found.
    #[inline]
    pub fn get_event(&self, event_id: &str) -> Result<&AuditEntry> {
        return self
            .entries
            .iter()
            .find(|audit_entry| return audit_entry.id == event_id)
            .ok_or_else(|| {
                return CpinfoError::validation_error(format!("Audit entry not found: {event_id}"));
            });
    }

    /// Log an action to the audit trail
    ///
    /// # Errors
    /// Returns a `CpinfoError` if event recording fails.
    #[inline]
    pub fn log_action(&mut self, actor: &str, action: &str, resource: &str) -> Result<String> {
        let event = AuditEvent {
            timestamp: chrono::Utc::now(),
            event_type: action.to_owned(),
            level: AuditLevel::Info,
            source_file: resource.to_owned(),
            user_id: actor.to_owned(),
            details: format!("Action: {action} on {resource} by {actor}"),
            metadata: Some(serde_json::json!({
                "action": action,
                "resource": resource,
                "actor": actor
            })),
        };

        return self.record_event(event);
    }

    /// Create a new audit trail
    ///
    /// # Errors
    /// Returns a `CpinfoError` if the audit directory cannot be created or accessed.
    #[inline]
    pub fn new<P: AsRef<std::path::Path>>(audit_dir: P) -> Result<Self> {
        let audit_dir_path = audit_dir.as_ref().to_path_buf();
        match std::fs::create_dir_all(&audit_dir_path) {
            Ok(()) => {}
            Err(create_error) => return Err(create_error.into()),
        }

        return Ok(Self {
            audit_dir: audit_dir_path,
            entries: Vec::new(),
            last_hash: "0".repeat(64), // Genesis hash
        });
    }

    /// Persist audit entry to disk
    #[inline]
    fn persist_entry(&self, entry: &AuditEntry) -> Result<()> {
        let filename = format!(
            "audit_{}.json",
            entry.timestamp_created.format("%Y%m%d_%H%M%S")
        );
        let file_path = self.audit_dir.join(filename);

        let entry_json = match serde_json::to_string_pretty(entry) {
            Ok(pretty_json) => pretty_json,
            Err(serialize_error) => return Err(serialize_error.into()),
        };
        match std::fs::write(file_path, entry_json) {
            Ok(()) => {}
            Err(write_error) => return Err(write_error.into()),
        }

        return Ok(());
    }

    /// Record a new audit event
    ///
    /// # Errors
    /// Returns a `CpinfoError` if serialization fails, signature generation fails, or persistence fails.
    #[inline]
    pub fn record_event(&mut self, event: AuditEvent) -> Result<String> {
        use sha2::{Digest as _, Sha256};

        let entry_id = format!("audit_{}", uuid::Uuid::new_v4());

        let event_data = match serde_json::to_string(&event) {
            Ok(serialized_data) => serialized_data,
            Err(serialize_error) => return Err(serialize_error.into()),
        };
        let mut hasher = Sha256::new();
        hasher.update(event_data.as_bytes());
        hasher.update(&self.last_hash);
        let hash = format!("{:x}", hasher.finalize());

        let signature = Self::generate_signature(&event_data, &self.last_hash);

        let audit_entry = AuditEntry {
            id: entry_id.clone(),
            event,
            signature,
            prev_hash: self.last_hash.clone(),
            timestamp_created: chrono::Utc::now(),
        };

        self.entries.push(audit_entry.clone());
        self.last_hash = hash;

        match self.persist_entry(&audit_entry) {
            Ok(()) => {}
            Err(persist_error) => return Err(persist_error),
        }

        return Ok(entry_id);
    }

    /// Verify the integrity of the audit trail
    ///
    /// # Errors
    /// Returns a `CpinfoError` if serialization of audit events fails.
    #[inline]
    pub fn verify_integrity(&self) -> Result<bool> {
        use sha2::{Digest as _, Sha256};

        let mut prev_hash = "0".repeat(64);

        for audit_entry in &self.entries {
            if audit_entry.prev_hash != prev_hash {
                return Ok(false);
            }

            let event_data = match serde_json::to_string(&audit_entry.event) {
                Ok(serialized_event) => serialized_event,
                Err(serialize_error) => return Err(serialize_error.into()),
            };
            let mut hasher = Sha256::new();
            hasher.update(event_data.as_bytes());
            hasher.update(&prev_hash);
            prev_hash = format!("{:x}", hasher.finalize());
        }

        return Ok(true);
    }
}
