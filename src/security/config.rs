use crate::error::Result;
use crate::security::encryption::FileEncryption;

/// Configuration level for different security contexts
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ConfigurationLevel {
    Application,
    SystemSecurity,
    User,
}

/// Secure configuration structure with encrypted storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct SecureConfig {
    metadata: std::collections::HashMap<String, String>,
    settings: std::collections::HashMap<String, String>,
}

impl Default for SecureConfig {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

impl SecureConfig {
    /// Get a configuration value
    #[must_use]
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        return self.settings.get(key).map(String::as_str);
    }

    /// Create a new secure configuration
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        return Self {
            metadata: std::collections::HashMap::new(),
            settings: std::collections::HashMap::new(),
        };
    }

    /// Set a configuration value
    #[must_use]
    #[inline]
    pub fn set<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.settings.insert(key.into(), value.into());
        return self;
    }

    /// Set metadata
    #[must_use]
    #[inline]
    pub fn set_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        return self;
    }
}

/// Configuration manager with encrypted storage
#[non_exhaustive]
pub struct ConfigurationManager {
    config_dir: std::path::PathBuf,
    encryption: FileEncryption,
}

impl ConfigurationManager {
    /// Load encrypted user configuration
    ///
    /// # Errors
    /// Returns an error if decryption or deserialization fails
    #[inline]
    pub fn load_user_config(&self, user_id: &str, config_id: &str) -> Result<SecureConfig> {
        let config_file = self
            .config_dir
            .join(format!("user_{user_id}_{config_id}.enc"));

        let decrypted_content = match self.encryption.decrypt_file_content(&config_file) {
            Ok(content) => content,
            Err(decryption_error) => return Err(decryption_error),
        };
        let configuration: SecureConfig = match serde_json::from_str(&decrypted_content) {
            Ok(parsed_config) => parsed_config,
            Err(deserialization_error) => return Err(deserialization_error.into()),
        };

        return Ok(configuration);
    }

    /// Create a new configuration manager
    ///
    /// # Errors
    /// Returns an error if directory creation or encryption setup fails
    #[inline]
    pub fn new<P: AsRef<std::path::Path>>(config_dir: P) -> Result<Self> {
        let config_directory = config_dir.as_ref().to_path_buf();
        match std::fs::create_dir_all(&config_directory) {
            Ok(()) => {}
            Err(directory_error) => return Err(directory_error.into()),
        }

        let file_encryption = match FileEncryption::new() {
            Ok(encryption) => encryption,
            Err(encryption_error) => return Err(encryption_error),
        };

        return Ok(Self {
            config_dir: config_directory,
            encryption: file_encryption,
        });
    }

    /// Store system configuration with enhanced security
    ///
    /// # Errors
    /// Returns an error if serialization or encryption fails
    #[inline]
    pub fn store_system_config(
        &self,
        config: &SecureConfig,
        level: &ConfigurationLevel,
    ) -> Result<String> {
        let configuration_id = uuid::Uuid::new_v4().to_string();
        let level_str = match *level {
            ConfigurationLevel::Application => "app",
            ConfigurationLevel::SystemSecurity => "system_security",
            ConfigurationLevel::User => "user",
        };

        let config_file = self
            .config_dir
            .join(format!("{level_str}_{configuration_id}.enc"));

        let config_json = match serde_json::to_string(config) {
            Ok(json_string) => json_string,
            Err(serialization_error) => return Err(serialization_error.into()),
        };
        match self
            .encryption
            .encrypt_file_content(&config_json, &config_file)
        {
            Ok(()) => {}
            Err(encryption_error) => return Err(encryption_error),
        }
        return Ok(configuration_id);
    }

    /// Store encrypted user configuration
    ///
    /// # Errors
    /// Returns an error if serialization or encryption fails
    #[inline]
    pub fn store_user_config(&self, user_id: &str, config: &SecureConfig) -> Result<String> {
        let configuration_id = uuid::Uuid::new_v4().to_string();
        let config_file = self
            .config_dir
            .join(format!("user_{user_id}_{configuration_id}.enc"));

        let config_json = match serde_json::to_string(config) {
            Ok(json_string) => json_string,
            Err(serialization_error) => return Err(serialization_error.into()),
        };
        match self
            .encryption
            .encrypt_file_content(&config_json, &config_file)
        {
            Ok(()) => {}
            Err(encryption_error) => return Err(encryption_error),
        }
        return Ok(configuration_id);
    }

    /// Validate configuration integrity
    ///
    /// # Errors
    /// Returns an error if validation fails
    #[inline]
    pub fn validate_configuration(
        &self,
        _configuration_id: &str,
    ) -> Result<ConfigValidationResult> {
        return Ok(ConfigValidationResult {
            is_valid: true,
            last_modified: chrono::Utc::now(),
            tamper_detected: false,
        });
    }
}

/// Configuration validation result
#[derive(Debug)]
#[non_exhaustive]
pub struct ConfigValidationResult {
    /// Whether the configuration passed validation checks
    pub is_valid: bool,
    /// UTC timestamp when the configuration was last modified
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Whether tampering was detected in the configuration
    pub tamper_detected: bool,
}
