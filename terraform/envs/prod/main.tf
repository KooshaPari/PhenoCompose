module "prod" {
  source = "../.."
  environment = "prod"
  aws_region  = "us-west-2"
}
