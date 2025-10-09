#!/bin/bash

# Comprehensive Rust Linting Analysis Script
# Organizes all linting issues by file with detailed reporting

set -euo pipefail

# Colors for output (disabled when piping)
if [ -t 1 ]; then
	RED='\033[0;31m'
	GREEN='\033[0;32m'
	YELLOW='\033[1;33m'
	BLUE='\033[0;34m'
	CYAN='\033[0;36m'
	BOLD='\033[1m'
	NC='\033[0m' # No Color
else
	RED=''
	GREEN=''
	YELLOW=''
	BLUE=''
	CYAN=''
	BOLD=''
	NC=''
fi

# Configuration
OUTPUT_DIR="lint_report"
BY_FILE_DIR="${OUTPUT_DIR}/by_file"
BY_SEVERITY_DIR="${OUTPUT_DIR}/by_severity"
BY_ERROR_CODE_DIR="${OUTPUT_DIR}/by_error_code"
BY_CATEGORY_DIR="${OUTPUT_DIR}/by_category"
BY_FIX_DIFFICULTY_DIR="${OUTPUT_DIR}/by_fix_difficulty"
BY_PRIORITY_DIR="${OUTPUT_DIR}/by_priority"
RAW_OUTPUT="${OUTPUT_DIR}/raw_clippy_output.txt"
JSON_OUTPUT="${OUTPUT_DIR}/clippy_output.json"
SUMMARY_FILE="${OUTPUT_DIR}/summary.txt"
STATS_FILE="${OUTPUT_DIR}/statistics.txt"
COMPARISON_FILE="${OUTPUT_DIR}/progress_comparison.txt"

# Clean lint_report directory on every run
echo -e "${YELLOW}Cleaning lint_report directory...${NC}"
if [ -d "${OUTPUT_DIR}" ]; then
	echo "  Removing existing: ${OUTPUT_DIR}"
	rm -rf "${OUTPUT_DIR}"
fi

# Create output directories
mkdir -p "${BY_FILE_DIR}"
mkdir -p "${BY_SEVERITY_DIR}"
mkdir -p "${BY_ERROR_CODE_DIR}"
mkdir -p "${BY_CATEGORY_DIR}"
mkdir -p "${BY_FIX_DIFFICULTY_DIR}"
mkdir -p "${BY_PRIORITY_DIR}"

echo -e "${BOLD}${BLUE}=== Rust Linting Analysis Tool ===${NC}"
echo -e "${CYAN}Output directory: ${OUTPUT_DIR}${NC}"
echo ""

# Step 1: Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
if ! command -v cargo &>/dev/null; then
	echo -e "${RED}Error: cargo not found${NC}"
	exit 1
fi

if ! cargo clippy --version &>/dev/null; then
	echo -e "${RED}Error: clippy not found. Install with: rustup component add clippy${NC}"
	exit 1
fi

# Step 2: Run clippy with all strict rules
echo -e "${YELLOW}Running cargo clippy with strict rules...${NC}"
echo "This may take a while..."

# Run clippy with both text and JSON output
# Using set +e to continue despite exit codes
set +e

cargo clippy \
	--all-targets \
	--all-features \
	--color=never \
	2>&1 | tee "${RAW_OUTPUT}"

CLIPPY_EXIT_CODE=$?

# Also get JSON output for better parsing
cargo clippy \
	--all-targets \
	--all-features \
	--message-format=json \
	>"${JSON_OUTPUT}" 2>&1

set -e

echo "Clippy exit code: ${CLIPPY_EXIT_CODE} (continuing regardless)"

echo -e "${GREEN}Clippy analysis complete${NC}"
echo ""

# Step 3: Parse and organize issues by file
echo -e "${YELLOW}Parsing issues by file...${NC}"

# Create a Python script for parsing (more reliable than bash for complex parsing)
cat <<'EOF' >"${OUTPUT_DIR}/parse_clippy.py"
#!/usr/bin/env python3
import json
import re
import sys
import os
from collections import defaultdict, Counter
from pathlib import Path
from datetime import datetime

# Clippy lint categories and their priority levels
LINT_CATEGORIES = {
    # Correctness - highest priority
    'clippy::correctness': {'priority': 1, 'category': 'correctness', 'description': 'Code that is outright wrong or useless'},
    'clippy::suspicious': {'priority': 1, 'category': 'correctness', 'description': 'Code that is most likely wrong or useless'},
    
    # Security - high priority  
    'clippy::nursery': {'priority': 2, 'category': 'security', 'description': 'New lints that are still under development'},
    'clippy::restriction': {'priority': 2, 'category': 'security', 'description': 'Lints which prevent the use of language and library features'},
    
    # Performance - medium-high priority
    'clippy::perf': {'priority': 3, 'category': 'performance', 'description': 'Code that can be written to run faster'},
    
    # Complexity - medium priority
    'clippy::complexity': {'priority': 4, 'category': 'complexity', 'description': 'Code that does something simple but in a complex way'},
    
    # Style - lower priority
    'clippy::style': {'priority': 5, 'category': 'style', 'description': 'Code that should be written in a more idiomatic way'},
    'clippy::pedantic': {'priority': 5, 'category': 'style', 'description': 'Lints which are rather strict or have occasional false positives'},
    
    # Default for unknown
    'unknown': {'priority': 6, 'category': 'unknown', 'description': 'Uncategorized lint'}
}

# Auto-fixable lints (those that rustc/clippy can fix automatically)
AUTO_FIXABLE_LINTS = {
    'clippy::redundant_field_names', 'clippy::redundant_static_lifetimes', 'clippy::needless_return',
    'clippy::single_match', 'clippy::redundant_closure', 'clippy::useless_format', 'clippy::len_zero',
    'clippy::collapsible_if', 'clippy::redundant_pattern_matching', 'clippy::unnecessary_wraps',
    'clippy::match_single_binding', 'clippy::redundant_else', 'clippy::explicit_iter_loop',
    'clippy::map_unwrap_or', 'clippy::implicit_return', 'clippy::needless_borrow', 'clippy::clone_on_copy',
    'clippy::useless_conversion', 'clippy::identity_op', 'clippy::zero_width_space', 'clippy::single_char_pattern',
    'clippy::string_lit_as_bytes', 'clippy::bytes_nth', 'clippy::get_unwrap', 'clippy::filter_next'
}

def get_lint_info(lint_code):
    """Get category, priority, and fix difficulty for a lint code"""
    # Extract the main category from the lint code
    for category_prefix, info in LINT_CATEGORIES.items():
        if lint_code.startswith(category_prefix):
            break
    else:
        info = LINT_CATEGORIES['unknown']
    
    is_auto_fixable = lint_code in AUTO_FIXABLE_LINTS
    fix_difficulty = 'auto' if is_auto_fixable else 'manual'
    
    return {
        'category': info['category'],
        'priority': info['priority'],
        'fix_difficulty': fix_difficulty,
        'description': info['description']
    }

def parse_raw_output(raw_file, output_dir):
    """Parse raw clippy output and organize by file"""
    
    by_file = defaultdict(list)
    by_severity = defaultdict(list)
    current_file = None
    current_issue = []
    issue_count = defaultdict(lambda: defaultdict(int))
    lint_codes = set()
    
    with open(raw_file, 'r') as f:
        lines = f.readlines()
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Detect error/warning start
        if line.startswith('error:') or line.startswith('warning:') or line.startswith('note:'):
            # Save previous issue if exists
            if current_issue and current_file:
                issue_text = ''.join(current_issue)
                by_file[current_file].append(issue_text)
                
                # Extract lint code from issue text
                lint_match = re.search(r'#\[(\w+)\(([^)]+)\)\]', issue_text)
                if not lint_match:
                    lint_match = re.search(r'(\w+::\w+)', issue_text)
                if lint_match:
                    lint_codes.add(lint_match.group(1) if lint_match.lastindex == 1 else lint_match.group(2))
                
                # Determine severity
                if issue_text.startswith('error:'):
                    by_severity['errors'].append(f"File: {current_file}\n{issue_text}")
                    issue_count[current_file]['errors'] += 1
                elif issue_text.startswith('warning:'):
                    by_severity['warnings'].append(f"File: {current_file}\n{issue_text}")
                    issue_count[current_file]['warnings'] += 1
                else:
                    by_severity['notes'].append(f"File: {current_file}\n{issue_text}")
                    issue_count[current_file]['notes'] += 1
            
            # Start new issue
            current_issue = [line]
            
            # Look for file location in next few lines
            for j in range(i+1, min(i+5, len(lines))):
                if ' --> ' in lines[j]:
                    match = re.search(r'--> ([^:]+):(\d+):(\d+)', lines[j])
                    if match:
                        current_file = match.group(1)
                    break
        
        elif current_issue:
            # Continue collecting lines for current issue
            if line.strip() and not line.startswith('error:') and not line.startswith('warning:'):
                current_issue.append(line)
            elif not line.strip():
                # Empty line might indicate end of issue
                if current_file:
                    issue_text = ''.join(current_issue)
                    by_file[current_file].append(issue_text)
                    
                    # Extract lint code from issue text
                    lint_match = re.search(r'#\[(\w+)\(([^)]+)\)\]', issue_text)
                    if not lint_match:
                        lint_match = re.search(r'(\w+::\w+)', issue_text)
                    if lint_match:
                        lint_codes.add(lint_match.group(1) if lint_match.lastindex == 1 else lint_match.group(2))
                    
                    # Determine severity
                    if issue_text.startswith('error:'):
                        by_severity['errors'].append(f"File: {current_file}\n{issue_text}")
                        issue_count[current_file]['errors'] += 1
                    elif issue_text.startswith('warning:'):
                        by_severity['warnings'].append(f"File: {current_file}\n{issue_text}")
                        issue_count[current_file]['warnings'] += 1
                    else:
                        by_severity['notes'].append(f"File: {current_file}\n{issue_text}")
                        issue_count[current_file]['notes'] += 1
                
                current_issue = []
                current_file = None
        
        i += 1
    
    # Save last issue if exists
    if current_issue and current_file:
        issue_text = ''.join(current_issue)
        by_file[current_file].append(issue_text)
        
        # Extract lint code from issue text
        lint_match = re.search(r'#\[(\w+)\(([^)]+)\)\]', issue_text)
        if not lint_match:
            lint_match = re.search(r'(\w+::\w+)', issue_text)
        if lint_match:
            lint_codes.add(lint_match.group(1) if lint_match.lastindex == 1 else lint_match.group(2))
        
        if issue_text.startswith('error:'):
            by_severity['errors'].append(f"File: {current_file}\n{issue_text}")
            issue_count[current_file]['errors'] += 1
        elif issue_text.startswith('warning:'):
            by_severity['warnings'].append(f"File: {current_file}\n{issue_text}")
            issue_count[current_file]['warnings'] += 1
        else:
            by_severity['notes'].append(f"File: {current_file}\n{issue_text}")
            issue_count[current_file]['notes'] += 1
    
    return by_file, by_severity, issue_count, lint_codes

def parse_json_output(json_file):
    """Parse JSON clippy output for more structured data"""
    
    by_file = defaultdict(list)
    issue_types = defaultdict(int)
    by_error_code = defaultdict(list)
    by_category = defaultdict(list)
    by_fix_difficulty = defaultdict(list)
    by_priority = defaultdict(list)
    
    try:
        with open(json_file, 'r') as f:
            for line in f:
                try:
                    msg = json.loads(line)
                    if 'message' in msg and msg['message'].get('spans'):
                        spans = msg['message']['spans']
                        if spans and 'file_name' in spans[0]:
                            file_name = spans[0]['file_name']
                            level = msg['message'].get('level', 'unknown')
                            code = msg['message'].get('code', {}).get('code', 'unknown')
                            message = msg['message'].get('message', '')
                            
                            issue_info = {
                                'file': file_name,
                                'level': level,
                                'code': code,
                                'message': message,
                                'line': spans[0].get('line_start', 0),
                                'column': spans[0].get('column_start', 0)
                            }
                            
                            by_file[file_name].append(issue_info)
                            issue_types[code] += 1
                            
                            # Get lint categorization
                            lint_info = get_lint_info(code)
                            issue_info.update(lint_info)
                            
                            # Organize by various criteria
                            by_error_code[code].append(issue_info)
                            by_category[lint_info['category']].append(issue_info)
                            by_fix_difficulty[lint_info['fix_difficulty']].append(issue_info)
                            by_priority[lint_info['priority']].append(issue_info)
                            
                except json.JSONDecodeError:
                    continue
    except FileNotFoundError:
        pass
    
    return by_file, issue_types, by_error_code, by_category, by_fix_difficulty, by_priority

def write_error_code_reports(by_error_code, output_dir):
    """Write reports organized by error code"""
    error_code_dir = output_dir / "by_error_code"
    
    # Sort by frequency
    sorted_codes = sorted(by_error_code.items(), key=lambda x: len(x[1]), reverse=True)
    
    for error_code, issues in sorted_codes:
        safe_filename = error_code.replace(':', '_').replace('::', '_')
        output_file = error_code_dir / f"{safe_filename}.txt"
        
        lint_info = get_lint_info(error_code)
        
        with open(output_file, 'w') as f:
            f.write(f"=== {error_code} ===\n")
            f.write(f"Category: {lint_info['category']}\n")
            f.write(f"Priority: {lint_info['priority']}\n")
            f.write(f"Fix Difficulty: {lint_info['fix_difficulty']}\n")
            f.write(f"Description: {lint_info['description']}\n")
            f.write(f"Total occurrences: {len(issues)}\n")
            f.write("=" * 60 + "\n\n")
            
            # Group by file
            by_file = defaultdict(list)
            for issue in issues:
                by_file[issue['file']].append(issue)
            
            for file_path, file_issues in sorted(by_file.items()):
                f.write(f"File: {file_path} ({len(file_issues)} occurrences)\n")
                f.write("-" * 50 + "\n")
                for issue in file_issues:
                    f.write(f"  Line {issue['line']}: {issue['message']}\n")
                f.write("\n")

def write_category_reports(by_category, output_dir):
    """Write reports organized by lint category"""
    category_dir = output_dir / "by_category"
    
    # Sort by priority (correctness first, then by issue count)
    priority_order = ['correctness', 'security', 'performance', 'complexity', 'style', 'unknown']
    sorted_categories = []
    for category in priority_order:
        if category in by_category:
            sorted_categories.append((category, by_category[category]))
    
    for category, issues in sorted_categories:
        output_file = category_dir / f"{category}.txt"
        
        with open(output_file, 'w') as f:
            f.write(f"=== {category.upper()} LINTS ===\n")
            f.write(f"Total issues: {len(issues)}\n\n")
            
            # Group by error code within category
            by_code = defaultdict(list)
            for issue in issues:
                by_code[issue['code']].append(issue)
            
            sorted_codes = sorted(by_code.items(), key=lambda x: len(x[1]), reverse=True)
            
            for code, code_issues in sorted_codes:
                lint_info = get_lint_info(code)
                f.write(f"{code} ({len(code_issues)} occurrences)\n")
                f.write(f"  Priority: {lint_info['priority']}\n")
                f.write(f"  Fix: {lint_info['fix_difficulty']}\n")
                f.write(f"  Description: {lint_info['description']}\n")
                
                # Show top files affected
                file_counts = Counter(issue['file'] for issue in code_issues)
                top_files = file_counts.most_common(5)
                f.write(f"  Top affected files:\n")
                for file_path, count in top_files:
                    f.write(f"    {file_path}: {count}\n")
                f.write("\n")

def write_priority_reports(by_priority, output_dir):
    """Write reports organized by priority"""
    priority_dir = output_dir / "by_priority"
    
    priority_names = {
        1: 'critical_correctness',
        2: 'high_security', 
        3: 'medium_performance',
        4: 'medium_complexity',
        5: 'low_style',
        6: 'unknown'
    }
    
    for priority_level, issues in sorted(by_priority.items()):
        priority_name = priority_names.get(priority_level, f'priority_{priority_level}')
        output_file = priority_dir / f"{priority_name}.txt"
        
        with open(output_file, 'w') as f:
            f.write(f"=== PRIORITY {priority_level} ({priority_name.upper()}) ===\n")
            f.write(f"Total issues: {len(issues)}\n\n")
            
            # Group by category within priority
            by_category = defaultdict(list)
            for issue in issues:
                by_category[issue['category']].append(issue)
            
            for category, cat_issues in sorted(by_category.items()):
                f.write(f"{category.upper()}: {len(cat_issues)} issues\n")
                
                # Show top error codes in this category
                code_counts = Counter(issue['code'] for issue in cat_issues)
                top_codes = code_counts.most_common(10)
                for code, count in top_codes:
                    f.write(f"  {code}: {count}\n")
                f.write("\n")

def write_fix_difficulty_reports(by_fix_difficulty, output_dir):
    """Write reports organized by fix difficulty"""
    difficulty_dir = output_dir / "by_fix_difficulty"
    
    for difficulty, issues in sorted(by_fix_difficulty.items()):
        output_file = difficulty_dir / f"{difficulty}.txt"
        
        with open(output_file, 'w') as f:
            f.write(f"=== {difficulty.upper()} FIXES ===\n")
            f.write(f"Total issues: {len(issues)}\n")
            
            if difficulty == 'auto':
                f.write("\nThese can be fixed automatically with 'cargo clippy --fix'\n")
            else:
                f.write("\nThese require manual intervention\n")
            
            f.write("=" * 60 + "\n\n")
            
            # Group by error code
            by_code = defaultdict(list)
            for issue in issues:
                by_code[issue['code']].append(issue)
            
            sorted_codes = sorted(by_code.items(), key=lambda x: len(x[1]), reverse=True)
            
            for code, code_issues in sorted_codes:
                lint_info = get_lint_info(code)
                f.write(f"{code} ({len(code_issues)} occurrences)\n")
                f.write(f"  Category: {lint_info['category']}\n")
                f.write(f"  Priority: {lint_info['priority']}\n")
                f.write(f"  Description: {lint_info['description']}\n")
                
                # Show distribution across files
                file_counts = Counter(issue['file'] for issue in code_issues)
                f.write(f"  Files affected: {len(file_counts)}\n")
                if len(file_counts) <= 5:
                    for file_path, count in file_counts.most_common():
                        f.write(f"    {file_path}: {count}\n")
                else:
                    for file_path, count in file_counts.most_common(3):
                        f.write(f"    {file_path}: {count}\n")
                    f.write(f"    ... and {len(file_counts) - 3} more files\n")
                f.write("\n")

def compare_with_previous_runs(output_dir):
    """Compare current results with previous runs to show progress"""
    comparison_file = output_dir / "progress_comparison.txt"
    
    # Look for previous statistics file (backup from last run)
    prev_stats_file = output_dir.parent / "lint_report_previous_stats.txt"
    
    if not prev_stats_file.exists():
        with open(comparison_file, 'w') as f:
            f.write("=== PROGRESS COMPARISON ===\n\n")
            f.write("No previous run statistics found. This may be the first run.\n")
        return
    
    # Save current stats as previous for next run
    current_stats_file = output_dir / "statistics.txt"
    if current_stats_file.exists():
        import shutil
        shutil.copy2(current_stats_file, output_dir.parent / "lint_report_previous_stats.txt")
    
    if not prev_stats_file.exists():
        with open(comparison_file, 'w') as f:
            f.write("=== PROGRESS COMPARISON ===\n\n")
            f.write("Previous statistics file found but could not be read.\n")
        return
    
    # Read current and previous statistics
    current_stats = {}
    prev_stats = {}
    
    try:
        with open(output_dir / "statistics.txt", 'r') as f:
            for line in f:
                if ':' in line:
                    key, value = line.split(':', 1)
                    try:
                        current_stats[key.strip()] = int(value.strip())
                    except ValueError:
                        pass
        
        with open(prev_stats_file, 'r') as f:
            for line in f:
                if ':' in line:
                    key, value = line.split(':', 1)
                    try:
                        prev_stats[key.strip()] = int(value.strip())
                    except ValueError:
                        pass
        
        with open(comparison_file, 'w') as f:
            f.write("=== PROGRESS COMPARISON ===\n\n")
            f.write("Previous run vs Current run\n\n")
            
            metrics = ['Total issues', 'Total errors', 'Total warnings', 'Total files with issues']
            
            for metric in metrics:
                current = current_stats.get(metric, 0)
                previous = prev_stats.get(metric, 0)
                change = current - previous
                
                if change == 0:
                    status = "→ No change"
                elif change > 0:
                    status = f"↑ Increased by {change}"
                else:
                    status = f"↓ Decreased by {abs(change)}"
                
                f.write(f"{metric:<30} {previous:>8} → {current:>8} {status}\n")
            
            # Calculate improvement percentage
            prev_total = prev_stats.get('Total issues', 0)
            curr_total = current_stats.get('Total issues', 0)
            
            if prev_total > 0:
                improvement = ((prev_total - curr_total) / prev_total) * 100
                if improvement > 0:
                    f.write(f"\n🎉 Overall improvement: {improvement:.1f}% reduction in issues!\n")
                elif improvement < 0:
                    f.write(f"\n⚠️ Overall regression: {abs(improvement):.1f}% increase in issues.\n")
                else:
                    f.write(f"\n→ No overall change in issue count.\n")
    
    except Exception as e:
        with open(comparison_file, 'w') as f:
            f.write("=== PROGRESS COMPARISON ===\n\n")
            f.write(f"Error comparing with previous run: {str(e)}\n")

def main():
    if len(sys.argv) < 2:
        print("Usage: parse_clippy.py <output_dir>")
        sys.exit(1)
    
    output_dir = Path(sys.argv[1])
    raw_file = output_dir / "raw_clippy_output.txt"
    json_file = output_dir / "clippy_output.json"
    by_file_dir = output_dir / "by_file"
    by_severity_dir = output_dir / "by_severity"
    
    print("Parsing clippy output...")
    
    # Parse raw output
    by_file, by_severity, issue_count, lint_codes = parse_raw_output(raw_file, output_dir)
    
    # Parse JSON output for additional details
    json_by_file, issue_types, by_error_code, by_category, by_fix_difficulty, by_priority = parse_json_output(json_file)
    
    print(f"Found {len(issue_types)} different error codes in {len(json_by_file)} files")
    
    # Write individual file reports (existing functionality)
    for file_path, issues in by_file.items():
        safe_filename = file_path.replace('/', '_').replace('\\', '_')
        output_file = by_file_dir / f"{safe_filename}.txt"
        
        with open(output_file, 'w') as f:
            f.write(f"=== Linting Issues for {file_path} ===\n")
            f.write(f"Total issues: {len(issues)}\n")
            f.write(f"Errors: {issue_count[file_path]['errors']}\n")
            f.write(f"Warnings: {issue_count[file_path]['warnings']}\n")
            f.write(f"Notes: {issue_count[file_path]['notes']}\n")
            f.write("=" * 60 + "\n\n")
            
            for issue in issues:
                f.write(issue)
                f.write("\n" + "-" * 40 + "\n\n")
    
    # Write severity-based reports (existing functionality)
    for severity, issues in by_severity.items():
        output_file = by_severity_dir / f"{severity}.txt"
        with open(output_file, 'w') as f:
            f.write(f"=== All {severity.upper()} ===\n")
            f.write(f"Total: {len(issues)}\n")
            f.write("=" * 60 + "\n\n")
            
            for issue in issues:
                f.write(issue)
                f.write("\n" + "=" * 60 + "\n\n")
    
    # Write new organizational reports
    print("Writing error code reports...")
    write_error_code_reports(by_error_code, output_dir)
    
    print("Writing category reports...")
    write_category_reports(by_category, output_dir)
    
    print("Writing priority reports...")
    write_priority_reports(by_priority, output_dir)
    
    print("Writing fix difficulty reports...")
    write_fix_difficulty_reports(by_fix_difficulty, output_dir)
    
    print("Comparing with previous runs...")
    compare_with_previous_runs(output_dir)
    
    # Generate enhanced summary
    summary_file = output_dir / "summary.txt"
    with open(summary_file, 'w') as f:
        f.write("=== COMPREHENSIVE LINTING SUMMARY ===\n\n")
        
        # Basic statistics
        sorted_files = sorted(issue_count.items(), 
                            key=lambda x: sum(x[1].values()), 
                            reverse=True)
        
        total_errors = sum(counts['errors'] for _, counts in sorted_files)
        total_warnings = sum(counts['warnings'] for _, counts in sorted_files)
        total_notes = sum(counts['notes'] for _, counts in sorted_files)
        total_issues = total_errors + total_warnings + total_notes
        
        f.write("OVERVIEW:\n")
        f.write("-" * 40 + "\n")
        f.write(f"Total files analyzed: {len(sorted_files)}\n")
        f.write(f"Total issues: {total_issues}\n")
        f.write(f"  - Errors: {total_errors}\n")
        f.write(f"  - Warnings: {total_warnings}\n")
        f.write(f"  - Notes: {total_notes}\n\n")
        
        # Priority breakdown
        if by_priority:
            f.write("PRIORITY BREAKDOWN:\n")
            f.write("-" * 40 + "\n")
            priority_names = {
                1: 'Critical (Correctness)',
                2: 'High (Security)', 
                3: 'Medium (Performance)',
                4: 'Medium (Complexity)',
                5: 'Low (Style)',
                6: 'Unknown'
            }
            for priority in sorted(by_priority.keys()):
                count = len(by_priority[priority])
                name = priority_names.get(priority, f'Priority {priority}')
                f.write(f"  {name}: {count}\n")
            f.write("\n")
        
        # Fix difficulty breakdown
        if by_fix_difficulty:
            f.write("FIX DIFFICULTY:\n")
            f.write("-" * 40 + "\n")
            for difficulty in sorted(by_fix_difficulty.keys()):
                count = len(by_fix_difficulty[difficulty])
                f.write(f"  {difficulty.capitalize()} fixes: {count}\n")
            f.write("\n")
        
        # Top error codes
        if issue_types:
            f.write("TOP 15 ERROR CODES:\n")
            f.write("-" * 40 + "\n")
            sorted_types = sorted(issue_types.items(), key=lambda x: x[1], reverse=True)[:15]
            for error_code, count in sorted_types:
                lint_info = get_lint_info(error_code)
                f.write(f"  {error_code:<35} {count:>5} ({lint_info['category']})\n")
            f.write("\n")
        
        # Files needing most attention
        f.write("FILES NEEDING ATTENTION:\n")
        f.write("-" * 80 + "\n")
        f.write(f"{'File':<50} {'Errors':>8} {'Warnings':>10} {'Notes':>8} {'Total':>8}\n")
        f.write("-" * 80 + "\n")
        
        for file_path, counts in sorted_files[:15]:
            errors = counts['errors']
            warnings = counts['warnings']
            notes = counts['notes']
            total = errors + warnings + notes
            
            # Truncate long file paths
            display_path = file_path if len(file_path) <= 50 else "..." + file_path[-47:]
            f.write(f"{display_path:<50} {errors:>8} {warnings:>10} {notes:>8} {total:>8}\n")
    
    # Generate enhanced statistics
    stats_file = output_dir / "statistics.txt"
    with open(stats_file, 'w') as f:
        f.write("=== COMPREHENSIVE LINTING STATISTICS ===\n\n")
        f.write(f"Total files with issues: {len(issue_count)}\n")
        f.write(f"Total errors: {total_errors}\n")
        f.write(f"Total warnings: {total_warnings}\n")
        f.write(f"Total notes: {total_notes}\n")
        f.write(f"Total issues: {total_issues}\n")
        f.write(f"Unique error codes: {len(issue_types)}\n\n")
        
        if issue_count:
            avg_issues = total_issues / len(issue_count)
            f.write(f"Average issues per file: {avg_issues:.2f}\n\n")
    
    print("✅ All reports generated successfully!")

if __name__ == "__main__":
    main()
EOF

# Make Python script executable
chmod +x "${OUTPUT_DIR}/parse_clippy.py"

# Run the Python parser
if command -v python3 &>/dev/null; then
	python3 "${OUTPUT_DIR}/parse_clippy.py" "${OUTPUT_DIR}"
	echo -e "${GREEN}Issue parsing complete${NC}"
else
	echo -e "${YELLOW}Warning: Python 3 not found. Skipping detailed parsing.${NC}"
fi

# Step 4: Run additional checks
echo ""
echo -e "${YELLOW}Running additional checks...${NC}"

# Format check
echo "Checking formatting..."
cargo fmt --check >"${OUTPUT_DIR}/fmt_check.txt" 2>&1 || echo "Formatting issues found (see fmt_check.txt)"

# Build check
echo "Checking compilation..."
cargo check --all-targets --all-features >"${OUTPUT_DIR}/build_check.txt" 2>&1 || echo "Build issues found (see build_check.txt)"

# Doc check
echo "Checking documentation..."
cargo doc --no-deps --all-features >"${OUTPUT_DIR}/doc_check.txt" 2>&1 || echo "Doc issues found (see doc_check.txt)"

# Step 5: Create convenience symlinks
ln -sf "${OUTPUT_DIR}/summary.txt" "lint_summary_latest.txt"

# Step 6: Display summary
echo ""
echo -e "${BOLD}${GREEN}=== Analysis Complete ===${NC}"
echo ""

if [ -f "${SUMMARY_FILE}" ]; then
	echo -e "${CYAN}Summary:${NC}"
	head -n 20 "${SUMMARY_FILE}"
	echo ""
	echo -e "${YELLOW}Full summary available at: ${SUMMARY_FILE}${NC}"
fi

if [ -f "${STATS_FILE}" ]; then
	echo ""
	echo -e "${CYAN}Statistics:${NC}"
	cat "${STATS_FILE}"
fi

echo ""
echo -e "${BOLD}${BLUE}Output files created:${NC}"
echo "  - Full report: ${RAW_OUTPUT}"
echo "  - JSON data: ${JSON_OUTPUT}"
echo "  - Summary: ${SUMMARY_FILE}"
echo "  - Statistics: ${STATS_FILE}"
echo "  - Progress comparison: ${COMPARISON_FILE}"
echo "  - By file: ${BY_FILE_DIR}/"
echo "  - By severity: ${BY_SEVERITY_DIR}/"
echo "  - By error code: ${BY_ERROR_CODE_DIR}/"
echo "  - By category: ${BY_CATEGORY_DIR}/"
echo "  - By fix difficulty: ${BY_FIX_DIFFICULTY_DIR}/"
echo "  - By priority: ${BY_PRIORITY_DIR}/"
echo ""
echo -e "${GREEN}Reports saved to: ${OUTPUT_DIR}/${NC}"
echo ""
echo -e "${BOLD}${CYAN}New organizational features:${NC}"
echo "  📊 Error code separation: Individual files for each clippy rule"
echo "  🎯 Category separation: Correctness, security, performance, complexity, style"
echo "  🔧 Fix difficulty: Auto-fixable vs manual intervention required"
echo "  ⚡ Priority levels: Critical to low priority systematic fixing order"
echo "  📈 Progress tracking: Compare with previous runs to measure improvement"
