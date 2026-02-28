# IAM role for Lambda execution
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
          aws_secretsmanager_secret.database_url.arn,
          aws_secretsmanager_secret.jwt_secret.arn
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
          "sqs:SendMessage",
          "sqs:ReceiveMessage",
          "sqs:DeleteMessage",
          "sqs:GetQueueAttributes"
        ]
        Resource = [
          aws_sqs_queue.tasks.arn
        ]
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

# Rust API Lambda function (no VPC — connects to Neon over public internet)
resource "aws_lambda_function" "api" {
  filename         = "${path.module}/../../backend/target/lambda/rankchoice-api/bootstrap.zip"
  function_name    = "rankchoice-api-${var.environment}"
  role             = aws_iam_role.lambda_execution.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  timeout          = 30
  memory_size      = 512
  source_code_hash = filebase64sha256("${path.module}/../../backend/target/lambda/rankchoice-api/bootstrap.zip")

  environment {
    variables = {
      RUST_LOG     = var.environment == "prod" ? "info" : "debug"
      DATABASE_URL = var.neon_database_url
      ENVIRONMENT  = var.environment
      JWT_SECRET   = random_password.jwt_secret.result
      FRONTEND_URL = "https://${var.domain_name}"
    }
  }

  tracing_config {
    mode = "Active"
  }

  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic,
    aws_cloudwatch_log_group.api,
  ]
}

resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.api.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.main.execution_arn}/*/*"
}

# CloudWatch log group
resource "aws_cloudwatch_log_group" "api" {
  name              = "/aws/lambda/rankchoice-api-${var.environment}"
  retention_in_days = var.environment == "prod" ? 30 : 7
}
