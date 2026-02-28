output "cloudfront_url" {
  value = "https://${aws_cloudfront_distribution.main.domain_name}"
}

output "cloudfront_distribution_id" {
  value = aws_cloudfront_distribution.main.id
}

output "api_gateway_url" {
  value = aws_apigatewayv2_stage.main.invoke_url
}

output "static_bucket" {
  value = aws_s3_bucket.static_assets.id
}

output "domain_url" {
  value = "https://${var.domain_name}"
}
