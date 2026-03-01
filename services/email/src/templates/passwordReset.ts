import { EmailTemplate } from '../config/email';

export interface PasswordResetData {
  resetUrl: string;
  userName?: string;
  expiresIn: string;
}

export function createPasswordResetTemplate(data: PasswordResetData): EmailTemplate {
  const greeting = data.userName ? `Hi ${data.userName}` : 'Hello';

  const subject = 'Reset your password - RankedChoice.me';

  const text = `
${greeting},

We received a request to reset your password for your RankedChoice.me account.

Click the link below to set a new password:
${data.resetUrl}

This link will expire in ${data.expiresIn}.

If you didn't request a password reset, you can safely ignore this email. Your password will remain unchanged.

The RankedChoice.me Team
  `.trim();

  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Your Password</title>
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
            color: #4f46e5;
            margin-bottom: 10px;
        }
        .action-button {
            display: inline-block;
            background-color: #4f46e5;
            color: white;
            padding: 14px 28px;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            margin: 20px 0;
            text-align: center;
        }
        .warning-box {
            background-color: #fef3c7;
            border-left: 4px solid #f59e0b;
            padding: 16px;
            margin: 20px 0;
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
            <h1>Reset Your Password</h1>
        </div>

        <p>${greeting},</p>

        <p>We received a request to reset your password for your RankedChoice.me account. Click the button below to set a new password:</p>

        <div style="text-align: center;">
            <a href="${data.resetUrl}" class="action-button">Reset Password</a>
        </div>

        <div class="warning-box">
            <strong>This link will expire in ${data.expiresIn}.</strong>
        </div>

        <div class="info-box">
            <p style="margin: 0;">If you didn't request a password reset, you can safely ignore this email. Your password will remain unchanged.</p>
        </div>

        <div class="info-box">
            <p style="margin: 0;">If the button doesn't work, copy and paste this link into your browser:</p>
            <p style="margin: 8px 0 0 0; word-break: break-all;">${data.resetUrl}</p>
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
