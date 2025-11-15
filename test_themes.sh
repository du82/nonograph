#!/bin/bash

# Theme API Test Script
# This script tests the theme-related API endpoints

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://localhost:8009"
TIMEOUT=5

echo "🎨 Testing Nonograph Theme API"
echo "================================"

# Function to test HTTP endpoint
test_endpoint() {
    local method=$1
    local url=$2
    local expected_status=$3
    local description=$4
    local data=$5

    echo -n "Testing: $description... "

    if [ "$method" = "POST" ] && [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -d "$data" \
            --connect-timeout $TIMEOUT \
            "$BASE_URL$url" 2>/dev/null)
    else
        response=$(curl -s -w "%{http_code}" \
            --connect-timeout $TIMEOUT \
            "$BASE_URL$url" 2>/dev/null)
    fi

    if [ $? -ne 0 ]; then
        echo -e "${RED}FAILED${NC} (Connection error)"
        return 1
    fi

    http_code=${response: -3}
    body=${response%???}

    if [ "$http_code" = "$expected_status" ]; then
        echo -e "${GREEN}PASSED${NC} (HTTP $http_code)"
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "  Response preview: ${body:0:100}..."
        fi
        return 0
    else
        echo -e "${RED}FAILED${NC} (Expected HTTP $expected_status, got HTTP $http_code)"
        if [ -n "$body" ]; then
            echo "  Response: $body"
        fi
        return 1
    fi
}

# Check if server is running
echo "Checking if server is running..."
if ! curl -s --connect-timeout 2 "$BASE_URL" > /dev/null; then
    echo -e "${YELLOW}Warning: Server doesn't seem to be running on $BASE_URL${NC}"
    echo "Please start the server with: cargo run"
    echo "Then run this test script again."
    exit 1
fi

echo -e "${GREEN}Server is running!${NC}"
echo

# Test 1: Get all themes
test_endpoint "GET" "/api/themes" "200" "GET /api/themes (get all themes)"

echo

# Test 2: Get specific theme (should exist)
test_endpoint "GET" "/api/themes/light" "200" "GET /api/themes/light (get light theme)"

echo

# Test 3: Get specific theme (should exist)
test_endpoint "GET" "/api/themes/dark" "200" "GET /api/themes/dark (get dark theme)"

echo

# Test 4: Get non-existent theme
test_endpoint "GET" "/api/themes/nonexistent" "404" "GET /api/themes/nonexistent (theme not found)"

echo

# Test 5: Validate valid theme
valid_theme_data='{
  "name": "Test Theme",
  "background": "#ffffff",
  "text": "#000000",
  "is_dark": false,
  "button_bg": "#333333",
  "button_hover": "#000000",
  "button_active": "#111111",
  "button_text": "#ffffff",
  "menu_bg": "#ffffff",
  "menu_text": "#333333",
  "menu_hover": "#f5f5f5",
  "menu_selected": "#e8e8e8",
  "menu_border": "#dddddd",
  "menu_shadow": "rgba(0, 0, 0, 0.15)"
}'

test_endpoint "POST" "/api/validate-theme" "200" "POST /api/validate-theme (valid theme)" "$valid_theme_data"

echo

# Test 6: Validate invalid theme (bad color)
invalid_theme_data='{
  "name": "Invalid Theme",
  "background": "#gggggg",
  "text": "#000000",
  "is_dark": false,
  "button_bg": "#333333",
  "button_hover": "#000000",
  "button_active": "#111111",
  "button_text": "#ffffff",
  "menu_bg": "#ffffff",
  "menu_text": "#333333",
  "menu_hover": "#f5f5f5",
  "menu_selected": "#e8e8e8",
  "menu_border": "#dddddd",
  "menu_shadow": "rgba(0, 0, 0, 0.15)"
}'

test_endpoint "POST" "/api/validate-theme" "200" "POST /api/validate-theme (invalid color)" "$invalid_theme_data"

echo

# Test 7: Test some specific themes that should exist
themes_to_test=("sepia" "rose" "peach" "mint" "sage" "burgundy" "rust" "slate")

echo "Testing specific themes:"
for theme in "${themes_to_test[@]}"; do
    test_endpoint "GET" "/api/themes/$theme" "200" "GET /api/themes/$theme"
done

echo
echo "🎨 Theme API Test Complete!"
echo "================================"

# Summary
echo "Test completed. Check the results above."
echo "If any tests failed, make sure:"
echo "1. The server is running (cargo run)"
echo "2. The Themes.toml file exists and is valid"
echo "3. All required themes are defined in Themes.toml"
