terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name (dev, preview, prod)"
  type        = string
}

module "cluster" {
  source      = "./modules/cluster"
  environment = var.environment
  region      = var.aws_region
}

module "networking" {
  source      = "./modules/networking"
  environment = var.environment
  region      = var.aws_region
}

output "cluster_endpoint" {
  value = module.cluster.endpoint
}
