variable "environment" { type = string }

# Stub: real module would create RDS / CloudSQL
output "endpoint" {
  value = "db.${var.environment}.example.com:5432"
}
