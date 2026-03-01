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
      name: process.env.FROM_NAME || 'RankedChoice.me',
      address: process.env.FROM_EMAIL || 'noreply@rankedchoice.me',
    },
  };

  if (process.env.NODE_ENV !== 'development' && (!config.auth.user || !config.auth.pass)) {
    throw new Error('SMTP authentication credentials are required');
  }

  return config;
}

export function getEmailTransporter(): Transporter {
  if (!transporter) {
    const config = createEmailConfig();

    const transportOptions: nodemailer.TransportOptions & Record<string, unknown> = {
      host: config.host,
      port: config.port,
      secure: config.secure,
    };

    if (config.auth.user && config.auth.pass) {
      transportOptions.auth = config.auth;
    }

    transporter = nodemailer.createTransport(transportOptions);
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