# RankChoice.me Infrastructure — Lambda + Neon + S3/CloudFront

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }

  backend "s3" {
    bucket         = "rankchoice-terraform-state"
    key            = "prod/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "rankchoice-terraform-locks"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "RankChoice"
      Environment = var.environment
      ManagedBy   = "Terraform"
    }
  }
}

data "aws_route53_zone" "main" {
  name = var.domain_name
}

# JWT secret
resource "random_password" "jwt_secret" {
  length  = 64
  special = false
}

# Secrets Manager — database URL
resource "aws_secretsmanager_secret" "database_url" {
  name = "rankchoice-database-url-${var.environment}"
}

resource "aws_secretsmanager_secret_version" "database_url" {
  secret_id     = aws_secretsmanager_secret.database_url.id
  secret_string = var.neon_database_url
}

# Secrets Manager — JWT secret
resource "aws_secretsmanager_secret" "jwt_secret" {
  name = "rankchoice-jwt-secret-${var.environment}"
}

resource "aws_secretsmanager_secret_version" "jwt_secret" {
  secret_id     = aws_secretsmanager_secret.jwt_secret.id
  secret_string = random_password.jwt_secret.result
}

# SQS queue for async tasks
resource "aws_sqs_queue" "tasks" {
  name                      = "rankchoice-tasks-${var.environment}"
  delay_seconds             = 0
  max_message_size          = 262144
  message_retention_seconds = 1209600
  receive_wait_time_seconds = 20

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.tasks_dlq.arn
    maxReceiveCount     = 3
  })
}

resource "aws_sqs_queue" "tasks_dlq" {
  name                      = "rankchoice-tasks-dlq-${var.environment}"
  message_retention_seconds = 1209600
}
