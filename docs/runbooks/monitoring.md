# Monitoring Runbook

## compose-build high error rate

1. Check `kubectl logs -n phenocompose -l app=composer --tail=200`
2. Check upstream adapter health: `curl -sf http://adapter:8080/healthz`
3. Check resource saturation: `kubectl top pods -n phenocompose`
4. If adapter is failing, check the specific error type in metrics
5. If OOM, increase memory limits in deployment manifest
