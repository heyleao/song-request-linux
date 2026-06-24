$ErrorActionPreference = "SilentlyContinue"

$Shutdown = "http://127.0.0.1:7384/api/shutdown"
try {
    Invoke-WebRequest -UseBasicParsing -Method POST -Uri $Shutdown -Headers @{ "x-song-request-action" = "shutdown" } -TimeoutSec 2 | Out-Null
    Write-Host "Encerramento solicitado. Aguarde alguns segundos."
    exit 0
} catch {
    Write-Host "O app nao respondeu no painel local. Se ele ainda estiver aberto, feche pelo Gerenciador de Tarefas."
    exit 1
}
