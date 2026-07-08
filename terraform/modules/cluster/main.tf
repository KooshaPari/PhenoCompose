variable "environment" { type = string }
variable "region" { type = string }

# Stub: real module would create EKS/GKE cluster
output "endpoint" {
  value = "https://k8s.${var.environment}.${var.region}.example.com"
}
