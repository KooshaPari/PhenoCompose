# PhenoCompose Infrastructure

Terraform-managed infrastructure for dev / preview / prod environments.

## Layout

```
terraform/
  modules/
    cluster/        # EKS / GKE / AKS cluster
    database/       # RDS / CloudSQL
    networking/     # VPC, subnets, NAT
  envs/
    dev/
    preview/
    prod/
  main.tf           # root module
```

## Provisioning

```bash
cd terraform/envs/dev
terraform init
terraform plan -out=tfplan
terraform apply tfplan
```

## Environment tiers

| Env | Replicas | Region | Backup | Notes |
|-----|----------|--------|---------|-------|
| dev | 1 | us-west-2 | daily | Auto-scaled, no SLA |
| preview | 2 | us-west-2 | hourly | Per-PR ephemeral |
| prod | 6+ | multi-region | continuous | 99.95% SLA |

See `terraform/main.tf` for the root module.
