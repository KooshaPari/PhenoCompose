variable "environment" { type = string }
variable "region" { type = string }

# Stub: real module would create VPC + subnets
output "vpc_id" {
  value = "vpc-${var.environment}-${var.region}"
}
