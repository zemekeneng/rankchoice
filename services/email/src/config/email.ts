import nodemailer, { Transporter } from 'nodemailer';

export interface EmailConfig {
  host: string;
  port: number;
  secure: boolean;
  auth: {
    user: string;
    pass: string;
  };
  from: {
    name: string;
    address: string;
  };
}

export interface EmailTemplate {
  subject: string;
  text: string;
  html: string;
}

export interface SendEmailOptions {
  to: string;
  template: EmailTemplate;
  data?: Record<string, any>;
}

export interface EmailResult {
  success: boolean;
  messageId?: string;
  error?: string;
}

let transporter: Transporter | null = null;

export function createEmailConfig(): EmailConfig {
  const config: EmailConfig = {
    host: process.env.SMTP_HOST || 'localhost',
    port: parseInt(process.env.SMTP_PORT || '587'),
    secure: process.env.SMTP_SECURE === 'true',
    auth: {
      user: process.env.SMTP_USER || '',
      pass: process.env.SMTP_PASS || '',
    },
    from: {
      name: process.env.FROM_NAME || 'RankChoice.app',
      address: process.env.FROM_EMAIL || 'noreply@rankchoice.app',
    },
  };

  // Validate required config
  if (!config.auth.user || !config.auth.pass) {
    throw new Error('SMTP authentication credentials are required');
  }

  return config;
}

export function getEmailTransporter(): Transporter {
  if (!transporter) {
    const config = createEmailConfig();
    
    // Create transporter based on environment
    if (process.env.NODE_ENV === 'development') {
      // Use Ethereal Email for development testing
      transporter = nodemailer.createTransport({
        host: 'smtp.ethereal.email',
        port: 587,
        secure: false,
        auth: {
          user: process.env.ETHEREAL_USER || '',
          pass: process.env.ETHEREAL_PASS || '',
        },
      });
    } else {
      // Use configured SMTP for production
      transporter = nodemailer.createTransport({
        host: config.host,
        port: config.port,
        secure: config.secure,
        auth: config.auth,
      });
    }
  }

  return transporter!;
}

export async function testEmailConnection(): Promise<boolean> {
  try {
    const transporter = getEmailTransporter();
    await transporter.verify();
    console.log('✅ Email server connection verified');
    return true;
  } catch (error) {
    console.error('❌ Email server connection failed:', error);
    return false;
  }
}