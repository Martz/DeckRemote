#!/bin/bash
# Test script for DeckRemote API
# Usage: ./test-api.sh [SERVER_URL]
# Example: ./test-api.sh http://192.168.1.100:7394

SERVER_URL="${1:-http://localhost:7394}"

echo "Testing DeckRemote API at $SERVER_URL"
echo "========================================"
echo

# Test 1: Get initial state
echo "Test 1: Getting initial state..."
curl -s -X GET "$SERVER_URL/state" | jq '.' || echo "Error: Failed to get state"
echo
echo

# Test 2: Toggle LWIN key (block)
echo "Test 2: Toggling LWIN key (should block)..."
curl -s -X POST "$SERVER_URL/keys/toggle" \
  -H "Content-Type: application/json" \
  -d '{"key":"LWIN"}' | jq '.' || echo "Error: Failed to toggle key"
echo
echo

# Test 3: Get state (should show LWIN blocked)
echo "Test 3: Getting state (should show LWIN blocked)..."
curl -s -X GET "$SERVER_URL/state" | jq '.' || echo "Error: Failed to get state"
echo
echo

# Test 4: Toggle LWIN key (unblock)
echo "Test 4: Toggling LWIN key again (should unblock)..."
curl -s -X POST "$SERVER_URL/keys/toggle" \
  -H "Content-Type: application/json" \
  -d '{"key":"LWIN"}' | jq '.' || echo "Error: Failed to toggle key"
echo
echo

# Test 5: Get final state (should be empty)
echo "Test 5: Getting final state (should be empty)..."
curl -s -X GET "$SERVER_URL/state" | jq '.' || echo "Error: Failed to get state"
echo
echo

# Test 6: Toggle multiple keys
echo "Test 6: Blocking multiple keys..."
curl -s -X POST "$SERVER_URL/keys/toggle" \
  -H "Content-Type: application/json" \
  -d '{"key":"LWIN"}' | jq '.' || echo "Error: Failed to toggle LWIN"
echo

curl -s -X POST "$SERVER_URL/keys/toggle" \
  -H "Content-Type: application/json" \
  -d '{"key":"RWIN"}' | jq '.' || echo "Error: Failed to toggle RWIN"
echo

curl -s -X POST "$SERVER_URL/keys/toggle" \
  -H "Content-Type: application/json" \
  -d '{"key":"APPS"}' | jq '.' || echo "Error: Failed to toggle APPS"
echo
echo

# Test 7: Get state with multiple keys
echo "Test 7: Getting state with multiple keys..."
curl -s -X GET "$SERVER_URL/state" | jq '.' || echo "Error: Failed to get state"
echo
echo

echo "========================================"
echo "API tests complete!"
