# RankChoice.app Infrastructure Configuration

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

# Variables
variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "domain_name" {
  description = "Domain name for the application"
  type        = string
  default     = "rankchoice.app"
}

# Data sources
data "aws_route53_zone" "main" {
  name = var.domain_name
}

# VPC Configuration
module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "rankchoice-${var.environment}"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
  database_subnets = ["10.0.201.0/24", "10.0.202.0/24", "10.0.203.0/24"]

  enable_nat_gateway = true
  single_nat_gateway = var.environment != "prod"
  enable_dns_hostnames = true
  enable_dns_support = true

  enable_flow_log                      = true
  create_flow_log_cloudwatch_iam_role  = true
  create_flow_log_cloudwatch_log_group = true

  tags = {
    Name = "rankchoice-${var.environment}"
  }
}

# Security Groups
resource "aws_security_group" "lambda" {
  name_prefix = "rankchoice-lambda-"
  vpc_id      = module.vpc.vpc_id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "rankchoice-lambda-${var.environment}"
  }
}

resource "aws_security_group" "aurora" {
  name_prefix = "rankchoice-aurora-"
  vpc_id      = module.vpc.vpc_id

  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.lambda.id]
  }

  tags = {
    Name = "rankchoice-aurora-${var.environment}"
  }
}

# Aurora Serverless v2
resource "random_password" "db_password" {
  length  = 32
  special = true
}

resource "aws_secretsmanager_secret" "db_credentials" {
  name = "rankchoice-db-credentials-${var.environment}"
}

resource "aws_secretsmanager_secret_version" "db_credentials" {
  secret_id = aws_secretsmanager_secret.db_credentials.id
  secret_string = jsonencode({
    username = "rankchoice"
    password = random_password.db_password.result
  })
}

resource "aws_rds_cluster" "main" {
  cluster_identifier      = "rankchoice-${var.environment}"
  engine                  = "aurora-postgresql"
  engine_mode            = "provisioned"
  engine_version         = "15.4"
  database_name          = "rankchoice"
  master_username        = "rankchoice"
  master_password        = random_password.db_password.result
  
  serverlessv2_scaling_configuration {
    max_capacity = var.environment == "prod" ? 16 : 2
    min_capacity = 0.5
  }

  vpc_security_group_ids = [aws_security_group.aurora.id]
  db_subnet_group_name   = aws_db_subnet_group.aurora.name
  
  backup_retention_period = var.environment == "prod" ? 30 : 7
  preferred_backup_window = "03:00-04:00"
  
  enabled_cloudwatch_logs_exports = ["postgresql"]
  
  skip_final_snapshot = var.environment != "prod"
  final_snapshot_identifier = var.environment == "prod" ? "rankchoice-final-snapshot-${timestamp()}" : null
  
  tags = {
    Name = "rankchoice-${var.environment}"
  }
}

resource "aws_rds_cluster_instance" "main" {
  count              = var.environment == "prod" ? 2 : 1
  identifier         = "rankchoice-${var.environment}-${count.index}"
  cluster_identifier = aws_rds_cluster.main.id
  instance_class     = "db.serverless"
  engine             = aws_rds_cluster.main.engine
  engine_version     = aws_rds_cluster.main.engine_version
}

resource "aws_db_subnet_group" "aurora" {
  name       = "rankchoice-${var.environment}"
  subnet_ids = module.vpc.database_subnets

  tags = {
    Name = "rankchoice-${var.environment}"
  }
}

# S3 Buckets
resource "aws_s3_bucket" "static_assets" {
  bucket = "rankchoice-static-${var.environment}"
}

resource "aws_s3_bucket_public_access_block" "static_assets" {
  bucket = aws_s3_bucket.static_assets.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "static_assets" {
  bucket = aws_s3_bucket.static_assets.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket" "uploads" {
  bucket = "rankchoice-uploads-${var.environment}"
}

resource "aws_s3_bucket_public_access_block" "uploads" {
  bucket = aws_s3_bucket.uploads.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "uploads" {
  bucket = aws_s3_bucket.uploads.id

  rule {
    id = "delete-old-uploads"
    status = "Enabled"
    
    expiration {
      days = 90
    }
  }
}

# CloudFront Distribution
resource "aws_cloudfront_origin_access_identity" "main" {
  comment = "rankchoice-${var.environment}"
}

resource "aws_acm_certificate" "main" {
  domain_name       = var.domain_name
  validation_method = "DNS"
  
  subject_alternative_names = [
    "*.${var.domain_name}",
    "www.${var.domain_name}"
  ]

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_route53_record" "cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.main.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = data.aws_route53_zone.main.zone_id
}

resource "aws_acm_certificate_validation" "main" {
  certificate_arn         = aws_acm_certificate.main.arn
  validation_record_fqdns = [for record in aws_route53_record.cert_validation : record.fqdn]
}

resource "aws_cloudfront_distribution" "main" {
  enabled             = true
  is_ipv6_enabled    = true
  comment            = "rankchoice-${var.environment}"
  default_root_object = "index.html"
  aliases            = [var.domain_name, "www.${var.domain_name}"]
  price_class        = var.environment == "prod" ? "PriceClass_All" : "PriceClass_100"

  origin {
    domain_name = aws_s3_bucket.static_assets.bucket_regional_domain_name
    origin_id   = "S3-static"

    s3_origin_config {
      origin_access_identity = aws_cloudfront_origin_access_identity.main.cloudfront_access_identity_path
    }
  }

  origin {
    domain_name = "${aws_api_gateway_rest_api.main.id}.execute-api.${var.aws_region}.amazonaws.com"
    origin_id   = "API-Gateway"
    origin_path = "/prod"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  default_cache_behavior {
    allowed_methods  = ["GET", "HEAD"]
    cached_methods   = ["GET", "HEAD"]
    target_origin_id = "S3-static"

    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }

    viewer_protocol_policy = "redirect-to-https"
    min_ttl                = 0
    default_ttl            = 3600
    max_ttl                = 86400
    compress               = true
  }

  ordered_cache_behavior {
    path_pattern     = "/api/*"
    allowed_methods  = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods   = ["GET", "HEAD", "OPTIONS"]
    target_origin_id = "API-Gateway"

    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Content-Type", "X-Forwarded-For"]
      
      cookies {
        forward = "all"
      }
    }

    viewer_protocol_policy = "https-only"
    min_ttl                = 0
    default_ttl            = 0
    max_ttl                = 0
  }

  custom_error_response {
    error_code         = 403
    response_code      = 200
    response_page_path = "/index.html"
  }

  custom_error_response {
    error_code         = 404
    response_code      = 200
    response_page_path = "/index.html"
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn = aws_acm_certificate_validation.main.certificate_arn
    ssl_support_method  = "sni-only"
  }

  tags = {
    Name = "rankchoice-${var.environment}"
  }
}

# API Gateway
resource "aws_api_gateway_rest_api" "main" {
  name        = "rankchoice-${var.environment}"
  description = "RankChoice API"
  
  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

resource "aws_api_gateway_deployment" "main" {
  rest_api_id = aws_api_gateway_rest_api.main.id
  stage_name  = var.environment

  lifecycle {
    create_before_destroy = true
  }
  
  depends_on = [
    aws_api_gateway_method.proxy,
    aws_api_gateway_integration.lambda_proxy
  ]
}

resource "aws_api_gateway_resource" "proxy" {
  rest_api_id = aws_api_gateway_rest_api.main.id
  parent_id   = aws_api_gateway_rest_api.main.root_resource_id
  path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "proxy" {
  rest_api_id   = aws_api_gateway_rest_api.main.id
  resource_id   = aws_api_gateway_resource.proxy.id
  http_method   = "ANY"
  authorization = "NONE"
}

# Lambda Functions
resource "aws_iam_role" "lambda_execution" {
  name = "rankchoice-lambda-execution-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_basic" {
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
  role       = aws_iam_role.lambda_execution.name
}

resource "aws_iam_role_policy_attachment" "lambda_vpc" {
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
  role       = aws_iam_role.lambda_execution.name
}

resource "aws_iam_role_policy" "lambda_policy" {
  name = "rankchoice-lambda-policy-${var.environment}"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue",
          "secretsmanager:DescribeSecret"
        ]
        Resource = [
          aws_secretsmanager_secret.db_credentials.arn
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = [
          "${aws_s3_bucket.uploads.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "ses:SendEmail",
          "ses:SendRawEmail"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "xray:PutTraceSegments",
          "xray:PutTelemetryRecords"
        ]
        Resource = "*"
      }
    ]
  })
}

# Main API Lambda (Rust)
resource "aws_lambda_function" "api" {
  filename         = "${path.module}/../../target/lambda/rankchoice-api.zip"
  function_name    = "rankchoice-api-${var.environment}"
  role            = aws_iam_role.lambda_execution.arn
  handler         = "bootstrap"
  runtime         = "provided.al2"
  timeout         = 30
  memory_size     = 512

  environment {
    variables = {
      RUST_LOG           = var.environment == "prod" ? "info" : "debug"
      DATABASE_URL       = "postgresql://${aws_rds_cluster.main.master_username}:${random_password.db_password.result}@${aws_rds_cluster.main.endpoint}:5432/${aws_rds_cluster.main.database_name}"
      ENVIRONMENT        = var.environment
      JWT_SECRET         = random_password.jwt_secret.result
      FRONTEND_URL       = "https://${var.domain_name}"
    }
  }

  vpc_config {
    subnet_ids         = module.vpc.private_subnets
    security_group_ids = [aws_security_group.lambda.id]
  }

  tracing_config {
    mode = "Active"
  }

  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic,
    aws_iam_role_policy_attachment.lambda_vpc,
  ]
}

resource "random_password" "jwt_secret" {
  length  = 64
  special = false
}

resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.api.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.main.execution_arn}/*/*"
}

resource "aws_api_gateway_integration" "lambda_proxy" {
  rest_api_id = aws_api_gateway_rest_api.main.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = aws_api_gateway_method.proxy.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.api.invoke_arn
}

# Email Service Lambda (Node.js)
resource "aws_lambda_function" "email_service" {
  filename         = "${path.module}/../../services/email/dist/email-service.zip"
  function_name    = "rankchoice-email-${var.environment}"
  role            = aws_iam_role.lambda_execution.arn
  handler         = "index.handler"
  runtime         = "nodejs18.x"
  timeout         = 60
  memory_size     = 256

  environment {
    variables = {
      ENVIRONMENT  = var.environment
      FROM_EMAIL   = "noreply@${var.domain_name}"
      FRONTEND_URL = "https://${var.domain_name}"
    }
  }

  vpc_config {
    subnet_ids         = module.vpc.private_subnets
    security_group_ids = [aws_security_group.lambda.id]
  }
}

# SQS Queue for async tasks
resource "aws_sqs_queue" "tasks" {
  name                       = "rankchoice-tasks-${var.environment}"
  delay_seconds             = 0
  max_message_size          = 262144
  message_retention_seconds = 1209600 # 14 days
  receive_wait_time_seconds = 20
  
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.tasks_dlq.arn
    maxReceiveCount     = 3
  })
}

resource "aws_sqs_queue" "tasks_dlq" {
  name = "rankchoice-tasks-dlq-${var.environment}"
  message_retention_seconds = 1209600 # 14 days
}

# CloudWatch Log Groups
resource "aws_cloudwatch_log_group" "api" {
  name              = "/aws/lambda/rankchoice-api-${var.environment}"
  retention_in_days = var.environment == "prod" ? 30 : 7
}

resource "aws_cloudwatch_log_group" "email_service" {
  name              = "/aws/lambda/rankchoice-email-${var.environment}"
  retention_in_days = var.environment == "prod" ? 30 : 7
}

# Route53 Records
resource "aws_route53_record" "main" {
  zone_id = data.aws_route53_zone.main.zone_id
  name    = var.domain_name
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "www" {
  zone_id = data.aws_route53_zone.main.zone_id
  name    = "www.${var.domain_name}"
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

# DynamoDB for session storage
resource "aws_dynamodb_table" "sessions" {
  name           = "rankchoice-sessions-${var.environment}"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "session_id"

  attribute {
    name = "session_id"
    type = "S"
  }

  ttl {
    attribute_name = "expires_at"
    enabled        = true
  }

  tags = {
    Name = "rankchoice-sessions-${var.environment}"
  }
}

# Outputs
output "cloudfront_url" {
  value = "https://${aws_cloudfront_distribution.main.domain_name}"
}

output "api_gateway_url" {
  value = aws_api_gateway_deployment.main.invoke_url
}

output "database_endpoint" {
  value = aws_rds_cluster.main.endpoint
}

output "static_bucket" {
  value = aws_s3_bucket.static_assets.id
} 