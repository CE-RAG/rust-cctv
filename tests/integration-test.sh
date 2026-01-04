#!/bin/bash

# Integration Test Script for CCTV Search API
# This script tests the /search endpoint with specific test cases

# Define the base URL for the API
BASE_URL="http://localhost:8080"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}CCTV Search API Integration Tests${NC}"
echo "======================================"

# Check if the server is running
echo -e "${YELLOW}Checking if server is running...${NC}"
if curl -s "${BASE_URL}" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is running${NC}"
else
    echo -e "${RED}❌ Server is not running. Please start the server first.${NC}"
    exit 1
fi

# Function to perform a search with provided JSON body
perform_search() {
    test_name=$1
    json_body=$2

    echo -e "\n${YELLOW}Test: $test_name${NC}"
    echo "--------------------------------------"
    echo "Request:"
    echo "$json_body" | jq .

    # Make the API call
    echo -e "\nResponse:"
    response=$(curl -s -X POST "${BASE_URL}/search" \
      -H "Content-Type: application/json" \
      -d "$json_body")

    # Pretty print the response
    echo "$response" | jq .

    # Check if the response is valid
    if echo "$response" | jq . > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Valid JSON response${NC}"
    else
        echo -e "${RED}❌ Invalid JSON response${NC}"
    fi

    echo "--------------------------------------"
}

# Test 1: Search with datetime filter
perform_search "Search with datetime filter" '{
  "query": "red sedan speeding",
  "top_k": 5,
  "start_date": "2025-10-08T06:00:00Z",
  "end_date": "2025-10-08T07:00:00Z"
}'

# Test 2: Search without datetime filter
perform_search "Search without datetime filter" '{
  "query": "red sedan speeding",
  "top_k": 10
}'

# Test 3: Different query with datetime filter
perform_search "Different query with datetime filter" '{
  "query": "motorcycle accident",
  "top_k": 3,
  "start_date": "2025-10-07T15:30:00Z",
  "end_date": "2025-10-07T16:30:00Z"
}'

# Test 4: Different query without datetime filter
perform_search "Different query without datetime filter" '{
  "query": "truck at intersection",
  "top_k": 15
}'

# Test 5: Search with only query (minimal parameters)
perform_search "Search with minimal parameters" '{
  "query": "yellow car"
}'

# Test 6: Search with only query and large top_k
perform_search "Search with large result set" '{
  "query": "vehicle",
  "top_k": 50
}'

# Test 7: Search with wide date range
perform_search "Search with wide date range" '{
  "query": "parking violation",
  "top_k": 8,
  "start_date": "2025-09-01T00:00:00Z",
  "end_date": "2025-10-15T23:59:59Z"
}'

echo -e "\n${YELLOW}Integration Tests Completed${NC}"
echo "======================================"
