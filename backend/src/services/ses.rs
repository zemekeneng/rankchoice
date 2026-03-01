use anyhow::{Context, Result};
use aws_sdk_ses::types::{Body, Content, Destination, Message};

#[derive(Clone)]
pub struct SesEmailSender {
    client: aws_sdk_ses::Client,
    from_address: String,
}

impl SesEmailSender {
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = aws_sdk_ses::Client::new(&config);

        let from_address = std::env::var("SES_FROM_ADDRESS")
            .unwrap_or_else(|_| "noreply@rankedchoice.me".to_string());

        Ok(Self {
            client,
            from_address,
        })
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<String> {
        let result = self
            .client
            .send_email()
            .source(&self.from_address)
            .destination(
                Destination::builder()
                    .to_addresses(to)
                    .build(),
            )
            .message(
                Message::builder()
                    .subject(Content::builder().data(subject).charset("UTF-8").build()?)
                    .body(
                        Body::builder()
                            .html(Content::builder().data(html_body).charset("UTF-8").build()?)
                            .text(Content::builder().data(text_body).charset("UTF-8").build()?)
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await
            .context("Failed to send email via SES")?;

        Ok(result.message_id().to_string())
    }

    pub async fn send_verification_email(
        &self,
        to: &str,
        user_name: Option<&str>,
        verification_url: &str,
    ) -> Result<String> {
        let greeting = user_name
            .map(|n| format!("Hi {}", n))
            .unwrap_or_else(|| "Hello".to_string());

        let subject = "Verify your email address - RankedChoice.me";

        let text = format!(
            "{greeting},\n\n\
            Thanks for signing up for RankedChoice.me! Please verify your email address by clicking the link below:\n\n\
            {verification_url}\n\n\
            This link will expire in 24 hours.\n\n\
            If you didn't create an account, you can safely ignore this email.\n\n\
            The RankedChoice.me Team"
        );

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #f8fafc; }}
.container {{ background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.07); }}
.header {{ text-align: center; margin-bottom: 30px; }}
.logo {{ font-size: 24px; font-weight: bold; color: #dc2626; }}
.btn {{ display: inline-block; background-color: #dc2626; color: white; padding: 14px 28px; text-decoration: none; border-radius: 8px; font-weight: 600; margin: 20px 0; }}
.info {{ background-color: #f3f4f6; padding: 16px; border-radius: 8px; margin: 20px 0; font-size: 14px; color: #6b7280; }}
.footer {{ text-align: center; margin-top: 30px; color: #6b7280; font-size: 14px; border-top: 1px solid #e5e7eb; padding-top: 20px; }}
</style></head>
<body><div class="container">
<div class="header"><div class="logo">RankedChoice.me</div><h1>Verify Your Email</h1></div>
<p>{greeting},</p>
<p>Thanks for signing up for RankedChoice.me! Please verify your email address by clicking the button below:</p>
<div style="text-align:center;"><a href="{verification_url}" style="display:inline-block;background-color:#dc2626;color:#ffffff;padding:14px 28px;text-decoration:none;border-radius:8px;font-weight:600;margin:20px 0;">Verify Email Address</a></div>
<div class="info"><p style="margin:0;">This link will expire in <strong>24 hours</strong>.</p><p style="margin:8px 0 0 0;">If you didn't create an account, you can safely ignore this email.</p></div>
<div class="info"><p style="margin:0;">If the button doesn't work, copy and paste this link into your browser:</p><p style="margin:8px 0 0 0;word-break:break-all;">{verification_url}</p></div>
<div class="footer"><p>This email was sent by RankedChoice.me</p></div>
</div></body></html>"#
        );

        self.send_email(to, subject, &html, &text).await
    }

    pub async fn send_password_reset_email(
        &self,
        to: &str,
        user_name: Option<&str>,
        reset_url: &str,
        expires_in: &str,
    ) -> Result<String> {
        let greeting = user_name
            .map(|n| format!("Hi {}", n))
            .unwrap_or_else(|| "Hello".to_string());

        let subject = "Reset your password - RankedChoice.me";

        let text = format!(
            "{greeting},\n\n\
            We received a request to reset your password for your RankedChoice.me account.\n\n\
            Click the link below to set a new password:\n\
            {reset_url}\n\n\
            This link will expire in {expires_in}.\n\n\
            If you didn't request a password reset, you can safely ignore this email.\n\n\
            The RankedChoice.me Team"
        );

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #f8fafc; }}
.container {{ background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.07); }}
.header {{ text-align: center; margin-bottom: 30px; }}
.logo {{ font-size: 24px; font-weight: bold; color: #dc2626; }}
.btn {{ display: inline-block; background-color: #dc2626; color: white; padding: 14px 28px; text-decoration: none; border-radius: 8px; font-weight: 600; margin: 20px 0; }}
.warning {{ background-color: #fef3c7; border-left: 4px solid #f59e0b; padding: 16px; margin: 20px 0; }}
.info {{ background-color: #f3f4f6; padding: 16px; border-radius: 8px; margin: 20px 0; font-size: 14px; color: #6b7280; }}
.footer {{ text-align: center; margin-top: 30px; color: #6b7280; font-size: 14px; border-top: 1px solid #e5e7eb; padding-top: 20px; }}
</style></head>
<body><div class="container">
<div class="header"><div class="logo">RankedChoice.me</div><h1>Reset Your Password</h1></div>
<p>{greeting},</p>
<p>We received a request to reset your password for your RankedChoice.me account. Click the button below to set a new password:</p>
<div style="text-align:center;"><a href="{reset_url}" style="display:inline-block;background-color:#dc2626;color:#ffffff;padding:14px 28px;text-decoration:none;border-radius:8px;font-weight:600;margin:20px 0;">Reset Password</a></div>
<div class="warning"><strong>This link will expire in {expires_in}.</strong></div>
<div class="info"><p style="margin:0;">If you didn't request a password reset, you can safely ignore this email.</p></div>
<div class="info"><p style="margin:0;">If the button doesn't work, copy and paste this link into your browser:</p><p style="margin:8px 0 0 0;word-break:break-all;">{reset_url}</p></div>
<div class="footer"><p>This email was sent by RankedChoice.me</p></div>
</div></body></html>"#
        );

        self.send_email(to, subject, &html, &text).await
    }
}
