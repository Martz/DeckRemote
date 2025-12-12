# Test script for DeckRemote API (PowerShell)
# Usage: .\test-api.ps1 [ServerUrl]
# Example: .\test-api.ps1 http://192.168.1.100:7394

param(
    [string]$ServerUrl = "http://localhost:7394"
)

Write-Host "Testing DeckRemote API at $ServerUrl" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

function Test-Api {
    param($Description, $Method, $Uri, $Body)
    
    Write-Host $Description -ForegroundColor Yellow
    try {
        $params = @{
            Uri = $Uri
            Method = $Method
            ContentType = "application/json"
        }
        
        if ($Body) {
            $params.Body = $Body
        }
        
        $response = Invoke-RestMethod @params
        $response | ConvertTo-Json
        Write-Host ""
    }
    catch {
        Write-Host "Error: $_" -ForegroundColor Red
        Write-Host ""
    }
}

# Test 1: Get initial state
Test-Api -Description "Test 1: Getting initial state..." `
    -Method GET `
    -Uri "$ServerUrl/state"

# Test 2: Toggle LWIN key (block)
Test-Api -Description "Test 2: Toggling LWIN key (should block)..." `
    -Method POST `
    -Uri "$ServerUrl/keys/toggle" `
    -Body '{"key":"LWIN"}'

# Test 3: Get state (should show LWIN blocked)
Test-Api -Description "Test 3: Getting state (should show LWIN blocked)..." `
    -Method GET `
    -Uri "$ServerUrl/state"

# Test 4: Toggle LWIN key (unblock)
Test-Api -Description "Test 4: Toggling LWIN key again (should unblock)..." `
    -Method POST `
    -Uri "$ServerUrl/keys/toggle" `
    -Body '{"key":"LWIN"}'

# Test 5: Get final state (should be empty)
Test-Api -Description "Test 5: Getting final state (should be empty)..." `
    -Method GET `
    -Uri "$ServerUrl/state"

# Test 6: Toggle multiple keys
Write-Host "Test 6: Blocking multiple keys..." -ForegroundColor Yellow

Test-Api -Description "  - Blocking LWIN..." `
    -Method POST `
    -Uri "$ServerUrl/keys/toggle" `
    -Body '{"key":"LWIN"}'

Test-Api -Description "  - Blocking RWIN..." `
    -Method POST `
    -Uri "$ServerUrl/keys/toggle" `
    -Body '{"key":"RWIN"}'

Test-Api -Description "  - Blocking APPS..." `
    -Method POST `
    -Uri "$ServerUrl/keys/toggle" `
    -Body '{"key":"APPS"}'

# Test 7: Get state with multiple keys
Test-Api -Description "Test 7: Getting state with multiple keys..." `
    -Method GET `
    -Uri "$ServerUrl/state"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "API tests complete!" -ForegroundColor Green
