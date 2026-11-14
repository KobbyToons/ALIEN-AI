@echo off
powershell -Command "Get-CimInstance -ClassName Win32_Battery | Select-Object EstimatedChargeRemaining, BatteryStatus | ConvertTo-Json"

