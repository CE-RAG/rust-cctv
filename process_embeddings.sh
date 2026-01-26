#!/bin/bash

# Configuration
API_URL="http://192.168.248.177:8003/predict"
INPUT_FILE="./file_paths.txt"
OUTPUT_FILE="embeddings_output.json"
BATCH_SIZE=16

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file '$INPUT_FILE' not found."
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: 'jq' is required but not installed. Please install jq first."
    exit 1
fi

# Read file paths into an array (compatible with older bash)
paths=()
while IFS= read -r line; do
    # Skip empty lines and comments
    [[ -n "$line" && ! "$line" =~ ^# ]] && paths+=("$line")
done < "$INPUT_FILE"
total_paths=${#paths[@]}

echo "Found $total_paths paths in $INPUT_FILE"
if [ $total_paths -gt 0 ]; then
    echo "First few paths:"
    for ((i=0; i<$(($total_paths<3?$total_paths:3)); i++)); do
        echo "  ${paths[$i]}"
    done
fi

echo "Processing $total_paths image paths in batches of $BATCH_SIZE..."

# Initialize output file with the correct structure
echo "{" > "$OUTPUT_FILE"
echo '  "type": "batch_image_embedding",' >> "$OUTPUT_FILE"
echo '  "results": [' >> "$OUTPUT_FILE"

batch_count=0
first_entry=true

# Process in batches
for ((i=0; i<$total_paths; i+=$BATCH_SIZE)); do
    batch_count=$((batch_count + 1))
    echo "Processing batch $batch_count..."

    # Get batch of paths
    batch_paths=("${paths[@]:$i:$BATCH_SIZE}")

    # Build JSON array for API request
    json_paths=$(printf ',"%s"' "${batch_paths[@]}")
    json_paths="[${json_paths:1}]"

    # Call API
    response=$(curl -s -X POST "$API_URL" \
        -H "Content-Type: application/json" \
        -d "{\"image_paths\": $json_paths}")

    # Check if response is valid JSON
    if ! echo "$response" | jq . &> /dev/null; then
        echo "Warning: Invalid JSON response from API for batch $batch_count"
        echo "$response" >> "${OUTPUT_FILE}.errors"
        continue
    fi

    # Parse and save results using a temporary file to avoid subshell issues
    temp_results=$(mktemp)
    echo "$response" | jq -r '.results[] | @base64' > "$temp_results"

    while read -r result; do
        # Decode base64 and extract values
        decoded=$(echo "$result" | base64 -d)
        path=$(echo "$decoded" | jq -r '.path')
        embedding=$(echo "$decoded" | jq -c '.embedding // null')
        error=$(echo "$decoded" | jq -r '.error // null')

        # Add comma before entry if not first
        if [ "$first_entry" = false ]; then
            echo "," >> "$OUTPUT_FILE"
        fi
        first_entry=false

        # Write entry
        if [ "$error" != "null" ]; then
            echo "    {" >> "$OUTPUT_FILE"
            echo "      \"path\": \"$path\"," >> "$OUTPUT_FILE"
            echo "      \"error\": \"$error\"" >> "$OUTPUT_FILE"
            echo -n "    }" >> "$OUTPUT_FILE"
        else
            echo "    {" >> "$OUTPUT_FILE"
            echo "      \"path\": \"$path\"," >> "$OUTPUT_FILE"
            echo "      \"embedding\": $embedding" >> "$OUTPUT_FILE"
            echo -n "    }" >> "$OUTPUT_FILE"
        fi
    done < "$temp_results"

    # Clean up temp file
    rm "$temp_results"

    # Small delay between batches
    sleep 0.5
done

# Close JSON structure
echo "" >> "$OUTPUT_FILE"
echo "  ]" >> "$OUTPUT_FILE"
echo "}" >> "$OUTPUT_FILE"

echo "Processing complete! Results saved to $OUTPUT_FILE"

# Optional: Validate the output file
if jq . "$OUTPUT_FILE" &> /dev/null; then
    echo "Output file is valid JSON."
else
    echo "Warning: Output file is not valid JSON."
fi
