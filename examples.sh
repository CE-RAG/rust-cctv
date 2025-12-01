#!/bin/bash

# Examples of using the CCTV Search Backend API
# Make sure the server is running on http://localhost:8080

BASE_URL="http://localhost:8080"

echo "========================================"
echo "🔍 CCTV Search Backend API Examples"
echo "========================================"

# 1. Insert an image with dash format URL
echo -e "\n1️⃣ Insert image (dash format URL):"
curl -X POST "$BASE_URL/insert_image" \
  -H "Content-Type: application/json" \
  -d '{"image": "https://i.postimg.cc/XNxQM7Z2/cctv08-2025-10-08-06-32-4.jpg"}' \
  | jq '.'

# 2. Insert an image with underscore format
echo -e "\n2️⃣ Insert image (underscore format):"
curl -X POST "$BASE_URL/insert_image" \
  -H "Content-Type: application/json" \
  -d '{"image": "cctv08_2025-10-08_06-32_4.jpg"}' \
  | jq '.'

# 3. Search without datetime filtering
echo -e "\n3️⃣ Search without datetime filter:"
curl -X POST "$BASE_URL/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "red car", "top_k": 3}' \
  | jq '.'

# 4. Search with empty datetime strings (equivalent to no filter)
echo -e "\n4️⃣ Search with empty datetime strings:"
curl -X POST "$BASE_URL/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "red car", "top_k": 3, "start_date": "", "end_date": ""}' \
  | jq '.'

# 5. Search with datetime range (morning)
echo -e "\n5️⃣ Search with datetime filter (morning):"
curl -X POST "$BASE_URL/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "vehicle", "top_k": 3, "start_date": "2025-10-08T06:00:00Z", "end_date": "2025-10-08T07:00:00Z"}' \
  | jq '.'

# 6. Search with datetime range (evening)
echo -e "\n6️⃣ Search with datetime filter (evening):"
curl -X POST "$BASE_URL/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "vehicle", "top_k": 3, "start_date": "2025-10-08T18:00:00Z", "end_date": "2025-10-08T19:00:00Z"}' \
  | jq '.'

# 7. Search with date-only format
echo -e "\n7️⃣ Search with date-only format:"
curl -X POST "$BASE_URL/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "vehicle", "top_k": 3, "start_date": "2025-10-08", "end_date": "2025-10-09"}' \
  | jq '.'

echo -e "\n========================================"
echo "✅ Examples completed"
echo "========================================"
