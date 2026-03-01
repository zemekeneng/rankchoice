import { EmailTemplate } from '../config/email';

export interface EmailVerificationData {
  verificationUrl: string;
  userName?: string;
}

export function createEmailVerificationTemplate(data: EmailVerificationData): EmailTemplate {
  const greeting = data.userName ? `Hi ${data.userName}` : 'Hello';

  const subject = 'Verify your email address - RankedChoice.me';

  const text = `
${greeting},

Thanks for signing up for RankedChoice.me! Please verify your email address by clicking the link below:

${data.verificationUrl}

This link will expire in 24 hours.

If you didn't create an account, you can safely ignore this email.

The RankedChoice.me Team
  `.trim();

  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify Your Email</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f8fafc;
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07);
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        .logo {
            font-size: 24px;
            font-weight: bold;
            color: #dc2626;
            margin-bottom: 10px;
        }
        .action-button {
            display: inline-block;
            background-color: #dc2626;
            color: white;
            padding: 14px 28px;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            margin: 20px 0;
            text-align: center;
        }
        .info-box {
            background-color: #f3f4f6;
            padding: 16px;
            border-radius: 8px;
            margin: 20px 0;
            font-size: 14px;
            color: #6b7280;
        }
        .footer {
            text-align: center;
            margin-top: 30px;
            color: #6b7280;
            font-size: 14px;
            border-top: 1px solid #e5e7eb;
            padding-top: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">RankedChoice.me</div>
            <h1>Verify Your Email</h1>
        </div>

        <p>${greeting},</p>

        <p>Thanks for signing up for RankedChoice.me! Please verify your email address by clicking the button below:</p>

        <div style="text-align: center;">
            <a href="${data.verificationUrl}" class="action-button">Verify Email Address</a>
        </div>

        <div class="info-box">
            <p style="margin: 0;">This link will expire in <strong>24 hours</strong>.</p>
            <p style="margin: 8px 0 0 0;">If you didn't create an account, you can safely ignore this email.</p>
        </div>

        <div class="info-box">
            <p style="margin: 0;">If the button doesn't work, copy and paste this link into your browser:</p>
            <p style="margin: 8px 0 0 0; word-break: break-all;">${data.verificationUrl}</p>
        </div>

        <div class="footer">
            <p>This email was sent by RankedChoice.me</p>
            <p>Secure, transparent, democratic voting made simple.</p>
        </div>
    </div>
</body>
</html>
  `.trim();

  return {
    subject,
    text,
    html
  };
}
