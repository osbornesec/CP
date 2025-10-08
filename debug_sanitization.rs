use cpinfo_parser::section_parser::sanitization;

fn main() {
    // Test the sanitization functions directly
    let command_name = "date";
    let file_path = "/etc/hosts";
    
    let cmd_filename = sanitization::command_output_filename(command_name);
    let file_filename = sanitization::file_output_filename(file_path);
    
    println!("Command '{}' -> '{}'", command_name, cmd_filename);
    println!("File '{}' -> '{}'", file_path, file_filename);
    
    // Test the sanitize functions
    let cmd_sanitized = sanitization::sanitize_command_name(command_name);
    let file_sanitized = sanitization::sanitize_file_path(file_path);
    
    println!("Command sanitized: '{}'", cmd_sanitized);
    println!("File sanitized: '{}'", file_sanitized);
}