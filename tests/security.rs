//! Security control tests
//!
//! Tests for Phase 3: Security and Privacy Controls (Tests 25-36)
//! These tests implement security controls to protect sensitive data in cpinfo files.

use cpinfo_parser::security::incident::{IncidentManager, IncidentSeverity, IncidentType};
use cpinfo_parser::security::SensitiveDataFilter;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

/// Test utility: Create cpinfo file with sensitive data for testing
fn create_sensitive_cpinfo_file() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".info").unwrap();

    // Write realistic cpinfo content with embedded sensitive data
    writeln!(file, "Check Point Support Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "General Info").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Version: R81.10 - Build 029").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Credentials Section").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "admin_password: SuperSecret123!").unwrap();
    writeln!(file, "api_key: sk-1234567890abcdef1234567890abcdef").unwrap();
    writeln!(file, "ssh_private_key: -----BEGIN RSA PRIVATE KEY-----").unwrap();
    writeln!(file, "MIIEpAIBAAKCAQEA1234567890abcdef...").unwrap();
    writeln!(file, "-----END RSA PRIVATE KEY-----").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Network Configuration").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Internal IP: 192.168.1.100").unwrap();
    writeln!(file, "Management IP: 10.0.0.5").unwrap();
    writeln!(file, "External IP: 203.0.113.42").unwrap();

    file.flush().unwrap();
    file
}

/// Test utility: Create cpinfo file with various credential patterns
fn create_credential_patterns_file() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".info").unwrap();

    writeln!(file, "Check Point Support Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Various Credentials").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "password=mysecret123").unwrap();
    writeln!(file, "api-key: AKIA1234567890ABCDEF").unwrap();
    writeln!(
        file,
        "secret_token: ghp_1234567890abcdef1234567890abcdef123456"
    )
    .unwrap();
    writeln!(file, "private_key_file: /etc/ssh/ssh_host_rsa_key").unwrap();
    writeln!(file, "cert_password: CertPass123").unwrap();
    writeln!(file, "database_url: postgresql://user:pass@localhost/db").unwrap();

    file.flush().unwrap();
    file
}

/// Test 25: Should identify and exclude sensitive credential sections
#[test]
fn test_should_identify_and_exclude_sensitive_credential_sections() {
    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Attempt to filter sensitive data
    let result = SensitiveDataFilter::filter_file_content(file_path);

    match result {
        Ok(filtered_content) => {
            // Verify sensitive data has been filtered/sanitized
            assert!(
                !filtered_content.contains("SuperSecret123!"),
                "Password should be filtered from content"
            );
            assert!(
                !filtered_content.contains("sk-1234567890abcdef"),
                "API key should be filtered from content"
            );
            assert!(
                !filtered_content.contains("BEGIN RSA PRIVATE KEY"),
                "Private key should be filtered from content"
            );

            // Verify non-sensitive content is preserved
            assert!(
                filtered_content.contains("Version: R81.10"),
                "Non-sensitive version info should be preserved"
            );
            assert!(
                filtered_content.contains("General Info"),
                "Section headers should be preserved"
            );
        }
        Err(e) => panic!(
            "Sensitive data filtering should succeed, but got error: {}",
            e
        ),
    }
}

/// Test utility: Create cpinfo file with mixed sensitivity levels
fn create_mixed_sensitivity_file() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".info").unwrap();

    writeln!(file, "Check Point Support Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Public Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Product: Check Point Gaia").unwrap();
    writeln!(file, "Version: R81.10").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Internal Configuration").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "hostname: cp-gateway-01").unwrap();
    writeln!(file, "domain: internal.company.com").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Confidential Settings").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "admin_user: john.doe@company.com").unwrap();
    writeln!(file, "last_login: 2024-01-15 10:30:45").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Restricted Credentials").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "admin_password: SecurePass123!").unwrap();
    writeln!(file, "api_secret: sk-abc123def456ghi789jkl").unwrap();
    writeln!(file, "==============================================").unwrap();

    file.flush().unwrap();
    file
}

/// Test 26: Should detect private key material in certificates  
#[test]
fn test_should_detect_private_key_material_in_certificates() {
    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Use detection method to identify sensitive patterns without filtering
    let filter = cpinfo_parser::security::SensitiveDataFilter::new().unwrap();
    let content = std::fs::read_to_string(file_path).unwrap();
    let detected_patterns = filter.detect_sensitive_patterns(&content);

    // Verify private key material is detected
    let private_key_matches: Vec<_> = detected_patterns
        .iter()
        .filter(|m| m.pattern_type == "private_key")
        .collect();

    assert!(
        !private_key_matches.is_empty(),
        "Should detect private key material in content"
    );

    // Verify the detected private key contains the expected pattern
    assert!(
        private_key_matches
            .iter()
            .any(|m| m.matched_text.contains("BEGIN RSA PRIVATE KEY")),
        "Should detect PEM private key headers"
    );

    // Verify password detection as well
    let password_matches: Vec<_> = detected_patterns
        .iter()
        .filter(|m| m.pattern_type == "password")
        .collect();

    assert!(
        !password_matches.is_empty(),
        "Should also detect password patterns"
    );
}

/// Test 27: Data classification and labeling system
#[test]
fn test_data_classification_and_labeling_system() {
    let test_file = create_mixed_sensitivity_file();
    let file_path = test_file.path();

    // Attempt to classify data sections by sensitivity level
    let result = cpinfo_parser::security::DataClassifier::classify_file_content(file_path);

    match result {
        Ok(classification_result) => {
            // Verify that different classification levels are detected
            assert!(
                classification_result.contains_public_data(),
                "Should detect public data sections"
            );
            assert!(
                classification_result.contains_internal_data(),
                "Should detect internal data sections"
            );
            assert!(
                classification_result.contains_confidential_data(),
                "Should detect confidential data sections"
            );
            assert!(
                classification_result.contains_restricted_data(),
                "Should detect restricted data sections"
            );

            // Verify specific classifications
            let public_sections = classification_result
                .get_sections_by_level(cpinfo_parser::security::ClassificationLevel::Public);
            assert!(
                public_sections
                    .iter()
                    .any(|s| s.name.contains("Public Information")),
                "Should classify 'Public Information' section as public"
            );

            let restricted_sections = classification_result
                .get_sections_by_level(cpinfo_parser::security::ClassificationLevel::Restricted);
            assert!(
                restricted_sections
                    .iter()
                    .any(|s| s.name.contains("Restricted Credentials")),
                "Should classify 'Restricted Credentials' section as restricted"
            );

            // Verify classification statistics
            assert!(
                classification_result.get_total_sections() >= 4,
                "Should classify at least 4 different sections"
            );
        }
        Err(e) => panic!("Data classification should succeed, but got error: {}", e),
    }
}

/// Test 28: Encryption of sensitive output files based on classification levels
#[test]
fn test_encryption_of_sensitive_output_files() {
    use cpinfo_parser::security::{ClassificationLevel, DataClassifier, FileEncryption};
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_mixed_sensitivity_file();
    let file_path = test_file.path();

    // Create temporary directory for output files
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path();

    // First classify the data
    let classification_result = DataClassifier::classify_file_content(file_path).unwrap();

    // Attempt to create encrypted output files based on classification
    let result = FileEncryption::encrypt_classified_sections(&classification_result, output_dir);

    match result {
        Ok(encryption_result) => {
            // Verify public data is not encrypted
            let public_file = output_dir.join("public_sections.txt");
            assert!(
                public_file.exists(),
                "Public data file should be created unencrypted"
            );

            let public_content = fs::read_to_string(&public_file).unwrap();
            assert!(
                public_content.contains("Check Point Gaia"),
                "Public file should contain readable content"
            );

            // Verify restricted data is encrypted
            let restricted_file = output_dir.join("restricted_sections.enc");
            assert!(
                restricted_file.exists(),
                "Restricted data should be encrypted and saved"
            );

            // Verify encrypted file is not readable as plain text
            let encrypted_content = fs::read(&restricted_file).unwrap();
            assert!(
                !encrypted_content.starts_with(b"admin_password"),
                "Encrypted file should not contain readable secrets"
            );

            // Verify encryption metadata
            assert!(
                encryption_result.has_encrypted_files(),
                "Should report that files were encrypted"
            );

            let encryption_summary = encryption_result.get_summary();
            assert!(
                encryption_summary.contains("AES-256"),
                "Should use AES-256 encryption"
            );
            assert!(
                encryption_summary.contains("Restricted"),
                "Should mention restricted data encryption"
            );

            // Verify different classification levels have appropriate handling
            assert!(
                encryption_result.get_file_count_by_level(ClassificationLevel::Public) > 0,
                "Should have public files"
            );
            assert!(
                encryption_result.get_file_count_by_level(ClassificationLevel::Restricted) > 0,
                "Should have encrypted restricted files"
            );
        }
        Err(e) => panic!("File encryption should succeed, but got error: {}", e),
    }
}

/// Test 29: Audit trail creation with cryptographic integrity protection
#[test]
fn test_audit_trail_creation_with_cryptographic_integrity() {
    use cpinfo_parser::security::{AuditEvent, AuditLevel, AuditTrail};
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Create temporary directory for audit logs
    let temp_dir = tempdir().unwrap();
    let audit_dir = temp_dir.path();

    // Create audit trail for security operations
    let mut audit_trail = AuditTrail::new(audit_dir).unwrap();

    // Record security events during processing
    let result = audit_trail.record_event(AuditEvent {
        timestamp: chrono::Utc::now(),
        event_type: "SENSITIVE_DATA_DETECTION".to_string(),
        level: AuditLevel::Warning,
        source_file: file_path.to_string_lossy().to_string(),
        user_id: "security_scanner".to_string(),
        details: "Detected passwords and API keys in cpinfo file".to_string(),
        metadata: Some(serde_json::json!({
            "patterns_found": ["password", "api_key", "private_key"],
            "severity": "HIGH",
            "action_required": "ENCRYPTION_RECOMMENDED"
        })),
    });

    match result {
        Ok(audit_entry_id) => {
            // Verify audit trail integrity
            assert!(
                audit_trail.verify_integrity().unwrap(),
                "Audit trail should have cryptographic integrity"
            );

            // Verify audit file was created
            let audit_files: Vec<_> = fs::read_dir(audit_dir)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().contains("audit"))
                .collect();

            assert!(!audit_files.is_empty(), "Should create audit log files");

            // Verify audit entry can be retrieved
            let retrieved_entry = audit_trail.get_event(&audit_entry_id).unwrap();
            assert_eq!(
                retrieved_entry.event.event_type, "SENSITIVE_DATA_DETECTION",
                "Should retrieve correct audit event"
            );
            assert_eq!(
                retrieved_entry.event.level,
                AuditLevel::Warning,
                "Should preserve audit level"
            );

            // Verify cryptographic signature exists
            assert!(
                retrieved_entry.has_valid_signature(),
                "Audit entry should have valid cryptographic signature"
            );

            // Test tamper detection
            let tamper_result = audit_trail.detect_tampering();
            match tamper_result {
                Ok(tamper_report) => {
                    assert!(
                        !tamper_report.has_tampering(),
                        "Should not detect tampering in valid audit trail"
                    );
                    assert!(
                        tamper_report.get_verified_entries_count() > 0,
                        "Should have verified audit entries"
                    );
                }
                Err(e) => panic!("Tamper detection should succeed, but got error: {}", e),
            }

            // Test audit trail export for compliance
            let export_result =
                audit_trail.export_for_compliance(audit_dir.join("audit_export.json"));
            match export_result {
                Ok(export_summary) => {
                    assert!(
                        export_summary.total_events > 0,
                        "Export should contain audit events"
                    );
                    assert!(
                        export_summary.integrity_verified,
                        "Export should verify integrity before export"
                    );

                    // Verify export file exists and contains valid JSON
                    let export_file = audit_dir.join("audit_export.json");
                    assert!(export_file.exists(), "Audit export file should be created");

                    let export_content = fs::read_to_string(&export_file).unwrap();
                    let _parsed_export: serde_json::Value =
                        serde_json::from_str(&export_content).unwrap();
                }
                Err(e) => panic!("Audit export should succeed, but got error: {}", e),
            }
        }
        Err(e) => panic!("Audit event recording should succeed, but got error: {}", e),
    }
}

/// Test 30: User access control and authentication framework
#[test]
fn test_user_access_control_and_authentication_framework() {
    use cpinfo_parser::security::{AuthenticationManager, AuthenticationResult, UserSession};
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Create temporary directory for authentication data
    let temp_dir = tempdir().unwrap();
    let auth_dir = temp_dir.path();

    // Initialize authentication manager with basic config
    let mut auth_manager = AuthenticationManager::new(auth_dir).unwrap();

    // Test successful authentication with valid credentials
    let auth_result =
        auth_manager.authenticate("admin", "SecurePassword123!", Some("192.168.1.100"));

    match auth_result {
        Ok(AuthenticationResult::Success(session)) => {
            // Verify session was created with proper security attributes
            assert!(session.is_valid(), "Authenticated session should be valid");
            assert_eq!(
                session.get_user_id(),
                "admin",
                "Session should contain correct user ID"
            );
            assert!(
                session.has_permission("file_access"),
                "Admin user should have file access permission"
            );
            assert!(
                session.has_permission("sensitive_data_read"),
                "Admin user should have sensitive data read permission"
            );

            // Test file access with authenticated session
            let access_result = auth_manager.authorize_file_access(&session, file_path, "read");

            match access_result {
                Ok(access_granted) => {
                    assert!(
                        access_granted.is_allowed(),
                        "Admin should be authorized to read cpinfo files"
                    );
                    assert!(
                        access_granted.has_audit_trail(),
                        "File access should be audited"
                    );

                    // Verify access logging
                    let access_logs = access_granted.get_access_logs();
                    assert!(!access_logs.is_empty(), "Should log file access attempts");
                    assert!(
                        access_logs
                            .iter()
                            .any(|log| log.contains("file_access_granted")),
                        "Should log successful file access"
                    );
                }
                Err(e) => panic!(
                    "File access authorization should succeed, but got error: {}",
                    e
                ),
            }

            // Test session expiration handling
            let session_status = auth_manager.validate_session(&session);
            match session_status {
                Ok(is_active) => {
                    assert!(is_active, "New session should be active");
                }
                Err(e) => panic!("Session validation should succeed, but got error: {}", e),
            }

            // Test session logout
            let logout_result = auth_manager.logout(&session);
            match logout_result {
                Ok(logout_success) => {
                    assert!(logout_success, "Session logout should succeed");

                    // Verify session is now invalid
                    let post_logout_status = auth_manager.validate_session(&session);
                    match post_logout_status {
                        Ok(is_active) => {
                            assert!(!is_active, "Session should be inactive after logout");
                        }
                        Err(_) => {
                            // Session validation failure after logout is also acceptable
                        }
                    }
                }
                Err(e) => panic!("Session logout should succeed, but got error: {}", e),
            }
        }
        Ok(AuthenticationResult::Failed(reason)) => {
            panic!(
                "Authentication should succeed with valid credentials, but failed: {}",
                reason
            );
        }
        Ok(AuthenticationResult::Locked(until)) => {
            panic!(
                "Account should not be locked for valid credentials, locked until: {:?}",
                until
            );
        }
        Err(e) => panic!("Authentication should succeed, but got error: {}", e),
    }

    // Test failed authentication with invalid credentials
    let failed_auth_result =
        auth_manager.authenticate("admin", "WrongPassword", Some("192.168.1.100"));

    match failed_auth_result {
        Ok(AuthenticationResult::Failed(reason)) => {
            assert!(
                reason.contains("invalid_credentials") || reason.contains("authentication_failed"),
                "Should provide clear failure reason for invalid credentials"
            );
        }
        Ok(AuthenticationResult::Success(_)) => {
            panic!("Authentication should fail with invalid credentials");
        }
        Ok(AuthenticationResult::Locked(_)) => {
            // Account lockout is acceptable for failed authentication
        }
        Err(e) => panic!(
            "Authentication attempt should complete, but got error: {}",
            e
        ),
    }

    // Test brute force protection
    let mut lockout_triggered = false;
    for attempt in 1..=6 {
        let brute_force_result = auth_manager.authenticate(
            "admin",
            &format!("InvalidPassword{}", attempt),
            Some("192.168.1.100"),
        );

        match brute_force_result {
            Ok(AuthenticationResult::Locked(_)) => {
                lockout_triggered = true;
                break;
            }
            Ok(AuthenticationResult::Failed(_)) => {
                // Continue with more attempts
            }
            _ => {}
        }
    }

    assert!(
        lockout_triggered,
        "Should trigger account lockout after multiple failed attempts"
    );
}

/// Test 31: Role-based permissions enforcement with enterprise roles
#[test]
fn test_role_based_permissions_enforcement() {
    use cpinfo_parser::security::{PermissionSet, RoleManager, UserRole};
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Create temporary directory for role configuration
    let temp_dir = tempdir().unwrap();
    let role_dir = temp_dir.path();

    // Initialize role manager with enterprise role definitions
    let mut role_manager = RoleManager::new(role_dir).unwrap();

    // Test Admin role - should have full access
    let admin_result =
        role_manager.create_user_with_role("admin_user", "AdminPass123!", UserRole::Admin);

    match admin_result {
        Ok(admin_session) => {
            // Admin should have access to all sensitive operations
            assert!(
                admin_session.has_permission("sensitive_data_read"),
                "Admin should have sensitive data read permission"
            );
            assert!(
                admin_session.has_permission("incident_response"),
                "Admin should have incident response permission"
            );
            assert!(
                admin_session.has_permission("configuration_write"),
                "Admin should have configuration write permission"
            );
            assert!(
                admin_session.has_permission("compliance_export"),
                "Admin should have compliance export permission"
            );

            // Test file access authorization with admin role
            let file_access = role_manager
                .authorize_role_based_access(&admin_session, file_path, "sensitive_read")
                .unwrap();

            assert!(
                file_access.is_allowed(),
                "Admin should be authorized for sensitive file access"
            );
        }
        Err(e) => panic!("Admin role creation should succeed, but got error: {}", e),
    }

    // Test Security Analyst role - limited access
    let analyst_result = role_manager.create_user_with_role(
        "security_analyst",
        "AnalystPass123!",
        UserRole::SecurityAnalyst,
    );

    match analyst_result {
        Ok(analyst_session) => {
            // Security Analyst should have read access but limited write access
            assert!(
                analyst_session.has_permission("sensitive_data_read"),
                "Security Analyst should have sensitive data read permission"
            );
            assert!(
                analyst_session.has_permission("audit_read"),
                "Security Analyst should have audit read permission"
            );
            assert!(
                !analyst_session.has_permission("configuration_write"),
                "Security Analyst should NOT have configuration write permission"
            );
            assert!(
                !analyst_session.has_permission("incident_containment"),
                "Security Analyst should NOT have incident containment permission"
            );

            // Test limited file access
            let file_access = role_manager
                .authorize_role_based_access(&analyst_session, file_path, "sensitive_read")
                .unwrap();

            assert!(
                file_access.is_allowed(),
                "Security Analyst should be authorized for sensitive file reading"
            );

            // Test denied write access
            let write_access = role_manager
                .authorize_role_based_access(&analyst_session, file_path, "configuration_write")
                .unwrap();

            assert!(
                !write_access.is_allowed(),
                "Security Analyst should NOT be authorized for configuration writing"
            );
        }
        Err(e) => panic!(
            "Security Analyst role creation should succeed, but got error: {}",
            e
        ),
    }

    // Test Incident Responder role - specialized permissions
    let responder_result = role_manager.create_user_with_role(
        "incident_responder",
        "ResponderPass123!",
        UserRole::IncidentResponder,
    );

    match responder_result {
        Ok(responder_session) => {
            // Incident Responder should have incident-specific permissions
            assert!(
                responder_session.has_permission("incident_response"),
                "Incident Responder should have incident response permission"
            );
            assert!(
                responder_session.has_permission("incident_containment"),
                "Incident Responder should have incident containment permission"
            );
            assert!(
                responder_session.has_permission("emergency_access"),
                "Incident Responder should have emergency access permission"
            );
            assert!(
                !responder_session.has_permission("compliance_export"),
                "Incident Responder should NOT have compliance export permission"
            );

            // Test emergency access
            let emergency_access = role_manager
                .authorize_emergency_access(&responder_session, "security_incident_containment")
                .unwrap();

            assert!(
                emergency_access.is_allowed(),
                "Incident Responder should have emergency access during incidents"
            );
        }
        Err(e) => panic!(
            "Incident Responder role creation should succeed, but got error: {}",
            e
        ),
    }

    // Test Standard User role - minimal access
    let user_result =
        role_manager.create_user_with_role("standard_user", "UserPass123!", UserRole::StandardUser);

    match user_result {
        Ok(user_session) => {
            // Standard User should have minimal permissions
            assert!(
                user_session.has_permission("file_read"),
                "Standard User should have basic file read permission"
            );
            assert!(
                !user_session.has_permission("sensitive_data_read"),
                "Standard User should NOT have sensitive data read permission"
            );
            assert!(
                !user_session.has_permission("incident_response"),
                "Standard User should NOT have incident response permission"
            );
            assert!(
                !user_session.has_permission("configuration_write"),
                "Standard User should NOT have configuration write permission"
            );

            // Test denied sensitive access
            let sensitive_access = role_manager
                .authorize_role_based_access(&user_session, file_path, "sensitive_read")
                .unwrap();

            assert!(
                !sensitive_access.is_allowed(),
                "Standard User should NOT be authorized for sensitive file access"
            );
        }
        Err(e) => panic!(
            "Standard User role creation should succeed, but got error: {}",
            e
        ),
    }

    // Test role hierarchy and permission inheritance
    let hierarchy_test = role_manager.test_role_hierarchy();
    match hierarchy_test {
        Ok(hierarchy_result) => {
            assert!(
                hierarchy_result.admin_inherits_all_permissions(),
                "Admin role should inherit all permissions from lower roles"
            );
            assert!(
                hierarchy_result.roles_have_distinct_permissions(),
                "Each role should have distinct permission sets"
            );
            assert!(
                hierarchy_result.no_privilege_escalation_paths(),
                "Should not allow privilege escalation between roles"
            );
        }
        Err(e) => panic!("Role hierarchy test should succeed, but got error: {}", e),
    }
}

/// Test 32: Secure configuration management with encrypted storage
#[test]
fn test_secure_configuration_management() {
    use cpinfo_parser::security::{ConfigurationLevel, ConfigurationManager, SecureConfig};
    use std::fs;
    use tempfile::tempdir;

    // Create temporary directory for configuration storage
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path();

    // Initialize configuration manager with encryption
    let mut config_manager = ConfigurationManager::new(config_dir).unwrap();

    // Test storing user preferences with encryption
    let user_preferences = SecureConfig::new()
        .set("theme", "dark")
        .set("language", "english")
        .set("log_level", "info")
        .set("session_timeout", "3600")
        .set("max_file_size", "100MB");

    let store_result = config_manager.store_user_config(
        "admin_user",
        user_preferences,
        ConfigurationLevel::UserPreferences,
    );

    match store_result {
        Ok(config_id) => {
            // Verify configuration was stored with encryption
            let stored_file = config_dir.join(format!("user_admin_user_{}.enc", config_id));
            assert!(
                stored_file.exists(),
                "Encrypted configuration file should be created"
            );

            // Verify file is actually encrypted (not readable as plain text)
            let encrypted_content = fs::read(&stored_file).unwrap();
            let content_string = String::from_utf8_lossy(&encrypted_content);
            assert!(
                !content_string.contains("dark") && !content_string.contains("english"),
                "Configuration file should be encrypted and not contain plain text values"
            );

            // Test retrieving and decrypting configuration
            let retrieve_result = config_manager.load_user_config("admin_user", &config_id);
            match retrieve_result {
                Ok(retrieved_config) => {
                    assert_eq!(
                        retrieved_config.get("theme").unwrap(),
                        "dark",
                        "Retrieved configuration should match stored values"
                    );
                    assert_eq!(
                        retrieved_config.get("language").unwrap(),
                        "english",
                        "Retrieved configuration should preserve all settings"
                    );
                    assert_eq!(
                        retrieved_config.get("session_timeout").unwrap(),
                        "3600",
                        "Retrieved configuration should maintain data types"
                    );
                }
                Err(e) => panic!(
                    "Configuration retrieval should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!("Configuration storage should succeed, but got error: {}", e),
    }

    // Test system configuration with higher security level
    let system_config = SecureConfig::new()
        .set("encryption_algorithm", "AES-256-GCM")
        .set("key_rotation_interval", "30")
        .set("audit_retention_days", "365")
        .set("max_failed_auth_attempts", "5")
        .set("session_encryption", "enabled");

    let system_store_result =
        config_manager.store_system_config(system_config, ConfigurationLevel::SystemSecurity);

    match system_store_result {
        Ok(system_config_id) => {
            // Verify system config uses stronger encryption
            let system_file = config_dir.join(format!("system_security_{}.enc", system_config_id));
            assert!(
                system_file.exists(),
                "System configuration should be stored encrypted"
            );

            // Test configuration validation
            let validation_result = config_manager.validate_system_config(&system_config_id);
            match validation_result {
                Ok(validation_report) => {
                    assert!(
                        validation_report.is_valid(),
                        "System configuration should pass validation"
                    );
                    assert!(
                        validation_report.encryption_strength_adequate(),
                        "System configuration should use adequate encryption"
                    );
                    assert!(
                        validation_report.no_security_vulnerabilities(),
                        "System configuration should not introduce security vulnerabilities"
                    );
                }
                Err(e) => panic!(
                    "System configuration validation should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!(
            "System configuration storage should succeed, but got error: {}",
            e
        ),
    }

    // Test configuration access control
    let access_test_result = config_manager.test_access_controls("standard_user");
    match access_test_result {
        Ok(access_report) => {
            assert!(
                access_report.can_read_user_config(),
                "Users should be able to read their own configuration"
            );
            assert!(
                !access_report.can_read_system_config(),
                "Standard users should NOT be able to read system configuration"
            );
            assert!(
                !access_report.can_modify_security_settings(),
                "Standard users should NOT be able to modify security settings"
            );
        }
        Err(e) => panic!(
            "Configuration access control test should succeed, but got error: {}",
            e
        ),
    }

    // Test configuration backup and recovery
    let backup_result = config_manager.create_encrypted_backup();
    match backup_result {
        Ok(backup_info) => {
            assert!(
                backup_info.includes_all_configurations(),
                "Backup should include all configuration files"
            );
            assert!(backup_info.is_encrypted(), "Backup should be encrypted");
            assert!(
                backup_info.has_integrity_verification(),
                "Backup should include integrity verification"
            );

            // Test recovery from backup
            let recovery_result = config_manager.restore_from_backup(&backup_info.backup_id);
            match recovery_result {
                Ok(recovery_report) => {
                    assert!(
                        recovery_report.all_configs_restored(),
                        "All configurations should be restored from backup"
                    );
                    assert!(
                        recovery_report.integrity_verified(),
                        "Restored configurations should have verified integrity"
                    );
                }
                Err(e) => panic!(
                    "Configuration recovery should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!("Configuration backup should succeed, but got error: {}", e),
    }

    // Test configuration versioning and rollback
    let version_result = config_manager.test_configuration_versioning();
    match version_result {
        Ok(version_report) => {
            assert!(
                version_report.supports_versioning(),
                "Configuration manager should support versioning"
            );
            assert!(
                version_report.can_rollback_changes(),
                "Should be able to rollback configuration changes"
            );
            assert!(
                version_report.maintains_audit_trail(),
                "Configuration changes should be audited"
            );
        }
        Err(e) => panic!(
            "Configuration versioning test should succeed, but got error: {}",
            e
        ),
    }
}

/// Test 33: Privacy protection and PII handling with GDPR compliance
#[test]
fn test_privacy_protection_and_gdpr_compliance() {
    use cpinfo_parser::security::{
        DataSubjectRequest, GDPRCompliance, PIIDetector, PrivacyManager,
    };
    use std::fs;
    use tempfile::tempdir;

    // Create test file with PII data
    let temp_dir = tempdir().unwrap();
    let privacy_dir = temp_dir.path();

    let mut pii_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(pii_file, "Check Point Support Information").unwrap();
    writeln!(pii_file, "==============================================").unwrap();
    writeln!(pii_file, "User Information").unwrap();
    writeln!(pii_file, "==============================================").unwrap();
    writeln!(pii_file, "admin_email: john.doe@company.com").unwrap();
    writeln!(pii_file, "phone_number: +1-555-123-4567").unwrap();
    writeln!(pii_file, "social_security: 123-45-6789").unwrap();
    writeln!(pii_file, "credit_card: 4532-1234-5678-9012").unwrap();
    writeln!(pii_file, "user_id: john.doe").unwrap();
    writeln!(pii_file, "ip_address: 192.168.1.100").unwrap();
    writeln!(pii_file, "session_id: sess_abc123def456").unwrap();
    writeln!(pii_file, "last_login: 2024-01-15 14:30:22").unwrap();
    pii_file.flush().unwrap();

    // Initialize privacy manager with GDPR compliance
    let mut privacy_manager = PrivacyManager::new(privacy_dir).unwrap();

    // Test PII detection with comprehensive patterns
    let pii_detection_result = privacy_manager.detect_pii_in_file(pii_file.path());

    match pii_detection_result {
        Ok(pii_report) => {
            // Verify detection of various PII types
            assert!(
                pii_report.contains_email_addresses(),
                "Should detect email addresses as PII"
            );
            assert!(
                pii_report.contains_phone_numbers(),
                "Should detect phone numbers as PII"
            );
            assert!(
                pii_report.contains_social_security_numbers(),
                "Should detect social security numbers as PII"
            );
            assert!(
                pii_report.contains_credit_card_numbers(),
                "Should detect credit card numbers as PII"
            );
            assert!(
                pii_report.contains_ip_addresses(),
                "Should detect IP addresses as PII"
            );

            // Verify PII classification levels
            let classification_summary = pii_report.get_classification_summary();
            assert!(
                classification_summary.has_high_sensitivity_pii(),
                "Should classify SSN and credit card as high sensitivity PII"
            );
            assert!(
                classification_summary.has_medium_sensitivity_pii(),
                "Should classify email and phone as medium sensitivity PII"
            );

            // Test PII anonymization
            let anonymization_result = privacy_manager.anonymize_detected_pii(&pii_report);
            match anonymization_result {
                Ok(anonymized_content) => {
                    assert!(
                        !anonymized_content.contains("john.doe@company.com"),
                        "Email addresses should be anonymized"
                    );
                    assert!(
                        !anonymized_content.contains("123-45-6789"),
                        "SSN should be anonymized"
                    );
                    assert!(
                        !anonymized_content.contains("4532-1234-5678-9012"),
                        "Credit card numbers should be anonymized"
                    );
                    assert!(
                        anonymized_content.contains("[ANONYMIZED_EMAIL]")
                            || anonymized_content.contains("[REDACTED_EMAIL]"),
                        "Should contain anonymization markers for emails"
                    );
                }
                Err(e) => panic!("PII anonymization should succeed, but got error: {}", e),
            }
        }
        Err(e) => panic!("PII detection should succeed, but got error: {}", e),
    }

    // Test GDPR compliance features
    let gdpr_compliance = GDPRCompliance::new(privacy_dir).unwrap();

    // Test data subject access request (Right to Access)
    let access_request = DataSubjectRequest::new()
        .request_type("access")
        .subject_identifier("john.doe@company.com")
        .requested_data_categories(vec!["personal_info", "usage_logs", "stored_files"]);

    let access_result = gdpr_compliance.process_data_subject_request(access_request);
    match access_result {
        Ok(access_response) => {
            assert!(
                access_response.contains_personal_data(),
                "Access request should return personal data if found"
            );
            assert!(
                access_response.includes_data_sources(),
                "Should specify where personal data was found"
            );
            assert!(
                access_response.has_completion_timestamp(),
                "Should timestamp the completion of access request"
            );
            assert!(
                access_response.is_gdpr_compliant(),
                "Access response should be GDPR compliant"
            );
        }
        Err(e) => panic!("GDPR access request should succeed, but got error: {}", e),
    }

    // Test data subject deletion request (Right to Erasure)
    let deletion_request = DataSubjectRequest::new()
        .request_type("deletion")
        .subject_identifier("john.doe@company.com")
        .deletion_scope("all_personal_data")
        .legal_basis("withdrawal_of_consent");

    let deletion_result = gdpr_compliance.process_data_subject_request(deletion_request);
    match deletion_result {
        Ok(deletion_response) => {
            assert!(
                deletion_response.data_deleted(),
                "Personal data should be deleted as requested"
            );
            assert!(
                deletion_response.has_deletion_audit_trail(),
                "Deletion should be audited for compliance"
            );
            assert!(
                deletion_response.maintains_legal_requirements(),
                "Should maintain data required by law even after deletion request"
            );

            // Verify data is actually removed
            let verification_search = privacy_manager.search_for_pii("john.doe@company.com");
            match verification_search {
                Ok(search_results) => {
                    assert!(
                        search_results.no_pii_found(),
                        "PII should no longer be found after deletion"
                    );
                }
                Err(e) => panic!(
                    "PII verification search should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!("GDPR deletion request should succeed, but got error: {}", e),
    }

    // Test data portability request (Right to Data Portability)
    let portability_request = DataSubjectRequest::new()
        .request_type("portability")
        .subject_identifier("jane.smith@company.com")
        .output_format("json")
        .include_metadata(true);

    let portability_result = gdpr_compliance.process_data_subject_request(portability_request);
    match portability_result {
        Ok(portability_response) => {
            assert!(
                portability_response.data_exported(),
                "Personal data should be exported in machine-readable format"
            );
            assert!(
                portability_response.includes_metadata(),
                "Export should include data creation and modification timestamps"
            );
            assert!(
                portability_response.is_structured_format(),
                "Export should be in structured, commonly used format"
            );
        }
        Err(e) => panic!(
            "GDPR portability request should succeed, but got error: {}",
            e
        ),
    }

    // Test privacy impact assessment
    let privacy_impact_result = gdpr_compliance.conduct_privacy_impact_assessment(pii_file.path());
    match privacy_impact_result {
        Ok(pia_report) => {
            assert!(
                pia_report.identifies_privacy_risks(),
                "Privacy impact assessment should identify risks"
            );
            assert!(
                pia_report.recommends_mitigation_measures(),
                "Should recommend measures to mitigate privacy risks"
            );
            assert!(
                pia_report.evaluates_necessity_and_proportionality(),
                "Should evaluate if data processing is necessary and proportional"
            );
            assert!(
                pia_report.documents_legal_basis(),
                "Should document legal basis for processing personal data"
            );
        }
        Err(e) => panic!(
            "Privacy impact assessment should succeed, but got error: {}",
            e
        ),
    }

    // Test consent management
    let consent_result = gdpr_compliance.test_consent_management();
    match consent_result {
        Ok(consent_report) => {
            assert!(
                consent_report.supports_granular_consent(),
                "Should support granular consent for different data processing purposes"
            );
            assert!(
                consent_report.allows_consent_withdrawal(),
                "Should allow easy withdrawal of consent"
            );
            assert!(
                consent_report.maintains_consent_records(),
                "Should maintain records of consent given and withdrawn"
            );
            assert!(
                consent_report.consent_is_freely_given(),
                "Should ensure consent is freely given, specific, informed, and unambiguous"
            );
        }
        Err(e) => panic!(
            "Consent management test should succeed, but got error: {}",
            e
        ),
    }
}

/// Test 34: Compliance framework integration for SOC2 and ISO27001
#[test]
fn test_compliance_framework_integration() {
    use cpinfo_parser::security::{
        ComplianceEvidence, ComplianceManager, ISO27001Framework, SOC2Framework,
    };
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Create temporary directory for compliance data
    let temp_dir = tempdir().unwrap();
    let compliance_dir = temp_dir.path();

    // Initialize compliance manager
    let mut compliance_manager = ComplianceManager::new(compliance_dir).unwrap();

    // Test SOC2 compliance evidence collection
    let soc2_result = compliance_manager.collect_soc2_evidence(file_path);

    match soc2_result {
        Ok(soc2_evidence) => {
            // Test SOC2 Trust Criteria - Security
            assert!(
                soc2_evidence.demonstrates_access_controls(),
                "Should demonstrate logical and physical access controls (CC6.1-CC6.3)"
            );
            assert!(
                soc2_evidence.shows_system_monitoring(),
                "Should show system monitoring and incident response (CC7.1-CC7.5)"
            );
            assert!(
                soc2_evidence.proves_data_classification(),
                "Should prove data classification and handling procedures"
            );

            // Test SOC2 Trust Criteria - Availability
            assert!(
                soc2_evidence.documents_system_availability(),
                "Should document system availability monitoring and capacity"
            );
            assert!(
                soc2_evidence.shows_backup_procedures(),
                "Should show backup and recovery procedures"
            );

            // Test SOC2 Trust Criteria - Confidentiality
            assert!(
                soc2_evidence.proves_data_encryption(),
                "Should prove data encryption in transit and at rest"
            );
            assert!(
                soc2_evidence.demonstrates_confidentiality_agreements(),
                "Should demonstrate confidentiality agreements and training"
            );

            // Generate SOC2 compliance report
            let soc2_report_result = compliance_manager.generate_soc2_report(&soc2_evidence);
            match soc2_report_result {
                Ok(soc2_report) => {
                    assert!(
                        soc2_report.addresses_all_trust_criteria(),
                        "SOC2 report should address all applicable trust criteria"
                    );
                    assert!(
                        soc2_report.includes_control_descriptions(),
                        "Should include detailed control descriptions and testing"
                    );
                    assert!(
                        soc2_report.has_management_assertions(),
                        "Should include management assertions about control effectiveness"
                    );
                }
                Err(e) => panic!(
                    "SOC2 report generation should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!(
            "SOC2 evidence collection should succeed, but got error: {}",
            e
        ),
    }

    // Test ISO27001 compliance evidence collection
    let iso27001_result = compliance_manager.collect_iso27001_evidence(file_path);

    match iso27001_result {
        Ok(iso_evidence) => {
            // Test Annex A controls - Information Security Management System
            assert!(
                iso_evidence.demonstrates_isms_implementation(),
                "Should demonstrate ISMS implementation (A.5 - Information security policies)"
            );
            assert!(
                iso_evidence.shows_risk_management(),
                "Should show risk assessment and treatment (A.8 - Asset management)"
            );

            // Test Annex A controls - Access Control
            assert!(
                iso_evidence.proves_access_control_management(),
                "Should prove access control management (A.9 - Access control)"
            );
            assert!(
                iso_evidence.documents_user_access_provisioning(),
                "Should document user access provisioning and review"
            );

            // Test Annex A controls - Cryptography
            assert!(
                iso_evidence.demonstrates_cryptographic_controls(),
                "Should demonstrate cryptographic controls (A.10 - Cryptography)"
            );
            assert!(
                iso_evidence.shows_key_management(),
                "Should show cryptographic key management procedures"
            );

            // Test Annex A controls - Incident Management
            assert!(
                iso_evidence.proves_incident_management(),
                "Should prove incident management procedures (A.16 - Information security incident management)"
            );
            assert!(
                iso_evidence.documents_incident_response(),
                "Should document incident response and recovery procedures"
            );

            // Generate ISO27001 compliance report
            let iso_report_result = compliance_manager.generate_iso27001_report(&iso_evidence);
            match iso_report_result {
                Ok(iso_report) => {
                    assert!(
                        iso_report.addresses_applicable_controls(),
                        "ISO27001 report should address all applicable Annex A controls"
                    );
                    assert!(
                        iso_report.includes_statement_of_applicability(),
                        "Should include Statement of Applicability (SOA)"
                    );
                    assert!(
                        iso_report.documents_risk_treatment(),
                        "Should document risk treatment decisions"
                    );
                    assert!(
                        iso_report.shows_continual_improvement(),
                        "Should show evidence of continual improvement"
                    );
                }
                Err(e) => panic!(
                    "ISO27001 report generation should succeed, but got error: {}",
                    e
                ),
            }
        }
        Err(e) => panic!(
            "ISO27001 evidence collection should succeed, but got error: {}",
            e
        ),
    }

    // Test automated compliance monitoring
    let monitoring_result = compliance_manager.setup_automated_monitoring();
    match monitoring_result {
        Ok(monitoring_config) => {
            assert!(
                monitoring_config.monitors_access_patterns(),
                "Should monitor access patterns for anomalies"
            );
            assert!(
                monitoring_config.tracks_configuration_changes(),
                "Should track configuration changes"
            );
            assert!(
                monitoring_config.alerts_on_compliance_deviations(),
                "Should alert on compliance deviations"
            );
            assert!(
                monitoring_config.generates_periodic_reports(),
                "Should generate periodic compliance reports"
            );
        }
        Err(e) => panic!(
            "Automated compliance monitoring setup should succeed, but got error: {}",
            e
        ),
    }
}

/// Test 35: Security event logging and real-time monitoring
#[test]
fn test_security_event_logging_and_monitoring() {
    use cpinfo_parser::security::{MonitoringRule, SecurityEvent, SecurityMonitor, ThreatDetector};
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let file_path = test_file.path();

    // Create temporary directory for monitoring data
    let temp_dir = tempdir().unwrap();
    let monitoring_dir = temp_dir.path();

    // Initialize security monitor
    let mut security_monitor = SecurityMonitor::new(monitoring_dir).unwrap();

    // Test security event detection and logging
    let event_detection_result =
        security_monitor.process_file_access(file_path, "admin_user", "sensitive_read");

    match event_detection_result {
        Ok(security_events) => {
            assert!(
                !security_events.is_empty(),
                "Should generate security events for sensitive file access"
            );

            // Check for different types of security events
            let sensitive_access_events: Vec<_> = security_events
                .iter()
                .filter(|e| e.event_type == "SENSITIVE_FILE_ACCESS")
                .collect();
            assert!(
                !sensitive_access_events.is_empty(),
                "Should log sensitive file access events"
            );

            let privilege_events: Vec<_> = security_events
                .iter()
                .filter(|e| e.event_type == "PRIVILEGE_ESCALATION_ATTEMPT")
                .collect();
            // Note: May or may not have privilege escalation events depending on access pattern

            // Verify event details
            for event in &security_events {
                assert!(
                    !event.timestamp.is_empty(),
                    "Security events should have timestamps"
                );
                assert!(
                    !event.user_id.is_empty(),
                    "Security events should include user identification"
                );
                assert!(
                    !event.source_ip.is_empty() || event.source_ip == "localhost",
                    "Security events should include source information"
                );
                assert!(
                    event.risk_score >= 0.0 && event.risk_score <= 10.0,
                    "Security events should have risk scores between 0-10"
                );
            }
        }
        Err(e) => panic!(
            "Security event detection should succeed, but got error: {}",
            e
        ),
    }

    // Test real-time threat detection
    let threat_detector = ThreatDetector::new().unwrap();

    // Simulate suspicious access patterns
    let suspicious_activities = vec![
        ("user1", "multiple_failed_auth", chrono::Utc::now()),
        ("user1", "access_after_hours", chrono::Utc::now()),
        ("user2", "bulk_data_download", chrono::Utc::now()),
        ("user2", "unusual_file_access", chrono::Utc::now()),
    ];

    for (user, activity_type, timestamp) in suspicious_activities {
        let detection_result = threat_detector.analyze_activity(user, activity_type, timestamp);
        match detection_result {
            Ok(threat_assessment) => {
                if activity_type.contains("failed_auth") || activity_type.contains("bulk_download")
                {
                    assert!(
                        threat_assessment.is_suspicious(),
                        "Should detect suspicious activities like failed auth and bulk downloads"
                    );
                    assert!(
                        threat_assessment.risk_score > 5.0,
                        "Suspicious activities should have elevated risk scores"
                    );
                }
            }
            Err(e) => panic!("Threat detection should succeed, but got error: {}", e),
        }
    }

    // Test monitoring rule configuration
    let rule_config_result = security_monitor.configure_monitoring_rules();
    match rule_config_result {
        Ok(rules) => {
            // Verify various types of monitoring rules are configured
            let has_auth_rules = rules.iter().any(|r| r.rule_type == "AUTHENTICATION");
            let has_access_rules = rules.iter().any(|r| r.rule_type == "FILE_ACCESS");
            let has_data_rules = rules.iter().any(|r| r.rule_type == "DATA_EXFILTRATION");
            let has_config_rules = rules.iter().any(|r| r.rule_type == "CONFIGURATION_CHANGE");

            assert!(
                has_auth_rules,
                "Should have authentication monitoring rules"
            );
            assert!(has_access_rules, "Should have file access monitoring rules");
            assert!(
                has_data_rules,
                "Should have data exfiltration monitoring rules"
            );
            assert!(
                has_config_rules,
                "Should have configuration change monitoring rules"
            );

            // Test rule effectiveness
            for rule in &rules {
                assert!(rule.is_active(), "Monitoring rules should be active");
                assert!(
                    rule.has_valid_criteria(),
                    "Monitoring rules should have valid detection criteria"
                );
                assert!(
                    rule.has_appropriate_response(),
                    "Monitoring rules should have appropriate response actions"
                );
            }
        }
        Err(e) => panic!(
            "Monitoring rule configuration should succeed, but got error: {}",
            e
        ),
    }

    // Test alert generation and escalation
    let alert_test_result = security_monitor.test_alert_system();
    match alert_test_result {
        Ok(alert_report) => {
            assert!(
                alert_report.supports_multiple_severity_levels(),
                "Alert system should support multiple severity levels"
            );
            assert!(
                alert_report.has_escalation_procedures(),
                "Should have defined escalation procedures"
            );
            assert!(
                alert_report.can_aggregate_related_events(),
                "Should be able to aggregate related security events"
            );
            assert!(
                alert_report.suppresses_duplicate_alerts(),
                "Should suppress duplicate alerts to prevent alert fatigue"
            );
        }
        Err(e) => panic!("Alert system test should succeed, but got error: {}", e),
    }
}

/// Test 36: Incident response automation and containment procedures
#[test]
fn test_incident_response_automation() {
    use cpinfo_parser::security::{
        ContainmentProcedure, IncidentManager, ResponseAction, SecurityIncident,
    };
    use std::fs;
    use tempfile::tempdir;

    let test_file = create_sensitive_cpinfo_file();
    let _file_path = test_file.path();

    // Create temporary directory for incident response
    let temp_dir = tempdir().unwrap();
    let _incident_dir = temp_dir.path();

    // Initialize incident manager
    let mut incident_manager = IncidentManager::new().unwrap();

    // Test incident detection and classification
    let test_result = incident_manager
        .test_incident_response(&IncidentType::SecurityBreach)
        .unwrap();

    // Verify incident response was properly triggered
    assert!(
        test_result.response_triggered,
        "Incident response should be triggered for security breach"
    );
    assert!(
        test_result.containment_actions_count > 0,
        "Should have containment actions for security breach"
    );
    assert!(
        test_result.response_time_seconds < 60,
        "Response should be triggered within reasonable time"
    );

    // Test malware incident response
    let malware_test_result = incident_manager
        .test_incident_response(&IncidentType::MalwareDetection)
        .unwrap();

    // Verify malware-specific response
    assert!(
        malware_test_result.response_triggered,
        "Malware incident response should be triggered"
    );
    assert!(
        malware_test_result.containment_actions_count > 0,
        "Should have containment actions for malware detection"
    );

    // Test incident escalation procedures with unauthorized access
    let escalation_test_result = incident_manager
        .test_incident_response(&IncidentType::UnauthorizedAccess)
        .unwrap();

    // Verify escalation response
    assert!(
        escalation_test_result.response_triggered,
        "Unauthorized access incident response should be triggered"
    );
    assert!(
        escalation_test_result.containment_actions_count > 0,
        "Should have containment actions for unauthorized access"
    );

    // Test data leak incident documentation and reporting
    let documentation_result = incident_manager
        .test_incident_response(&IncidentType::DataLeak)
        .unwrap();

    // Verify documentation response
    assert!(
        documentation_result.response_triggered,
        "Data leak incident response should be triggered"
    );
    assert!(
        documentation_result.containment_actions_count > 0,
        "Should have containment actions for data leak"
    );

    // Test system compromise incident recovery procedures
    let recovery_test_result = incident_manager
        .test_incident_response(&IncidentType::SystemCompromise)
        .unwrap();

    // Verify recovery response
    assert!(
        recovery_test_result.response_triggered,
        "System compromise incident response should be triggered"
    );
    assert!(
        recovery_test_result.containment_actions_count > 0,
        "Should have containment actions for system compromise"
    );
}
