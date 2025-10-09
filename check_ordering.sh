#!/bin/bash

# Check for arbitrary_source_item_ordering violations
echo "Checking for arbitrary_source_item_ordering violations..."

# Run clippy and extract file information
cargo clippy --message-format=json 2>/dev/null |
	jq -r 'select(.message and .message.code and .message.code.code == "arbitrary_source_item_ordering") | 
         "\(.message.spans[0].file_name):\(.message.spans[0].line_start): \(.message.message)"' |
	sort

echo ""
echo "Summary by file:"
cargo clippy --message-format=json 2>/dev/null |
	jq -r 'select(.message and .message.code and .message.code.code == "arbitrary_source_item_ordering") | 
         .message.spans[0].file_name' |
	sort | uniq -c | sort -nr
