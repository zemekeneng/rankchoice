import { getEmailTransporter, EmailResult, createEmailConfig } from '../config/email';
import { createVoterInvitationTemplate, VoterInvitationData } from '../templates/voterInvitation';
import { createPollResultsTemplate, PollResultsData } from '../templates/pollResults';
import { v4 as uuidv4 } from 'uuid';

export interface EmailTrackingInfo {
  id: string;
  to: string;
  subject: string;
  sentAt: Date;
  messageId?: string;
  status: 'sent' | 'failed' | 'pending';
  error?: string;
  retryCount: number;
}

export class EmailService {
  private trackingStore: Map<string, EmailTrackingInfo> = new Map();
  private config = createEmailConfig();

  async sendVoterInvitation(data: VoterInvitationData, to: string): Promise<EmailResult> {
    const template = createVoterInvitationTemplate(data);
    return this.sendEmail(to, template, 'voter_invitation');
  }

  async sendPollResults(data: PollResultsData, to: string): Promise<EmailResult> {
    const template = createPollResultsTemplate(data);
    return this.sendEmail(to, template, 'poll_results');
  }

  async sendBulkVoterInvitations(
    data: VoterInvitationData, 
    recipients: string[]
  ): Promise<{ success: EmailResult[]; failed: EmailResult[] }> {
    const results = await Promise.allSettled(
      recipients.map(email => this.sendVoterInvitation(data, email))
    );

    const success: EmailResult[] = [];
    const failed: EmailResult[] = [];

    results.forEach((result) => {
      if (result.status === 'fulfilled' && result.value.success) {
        success.push(result.value);
      } else {
        const errorResult: EmailResult = result.status === 'fulfilled' 
          ? result.value 
          : { success: false, error: 'Promise rejected' };
        failed.push(errorResult);
      }
    });

    return { success, failed };
  }

  private async sendEmail(
    to: string, 
    template: { subject: string; text: string; html: string },
    type: string
  ): Promise<EmailResult> {
    const trackingId = uuidv4();
    const trackingInfo: EmailTrackingInfo = {
      id: trackingId,
      to,
      subject: template.subject,
      sentAt: new Date(),
      status: 'pending',
      retryCount: 0
    };

    this.trackingStore.set(trackingId, trackingInfo);

    try {
      const transporter = getEmailTransporter();
      
      const mailOptions = {
        from: `${this.config.from.name} <${this.config.from.address}>`,
        to,
        subject: template.subject,
        text: template.text,
        html: template.html,
        headers: {
          'X-Tracking-ID': trackingId,
          'X-Email-Type': type,
        }
      };

      const info = await transporter.sendMail(mailOptions);
      
      // Update tracking info
      trackingInfo.status = 'sent';
      trackingInfo.messageId = info.messageId;
      this.trackingStore.set(trackingId, trackingInfo);

      console.log(`✅ Email sent successfully to ${to}`, {
        messageId: info.messageId,
        trackingId,
        type,
        subject: template.subject
      });

      // In development, log the preview URL
      if (process.env.NODE_ENV === 'development' && info.previewURL) {
        console.log(`📧 Preview URL: ${info.previewURL}`);
      }

      return {
        success: true,
        messageId: info.messageId
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      // Update tracking info
      trackingInfo.status = 'failed';
      trackingInfo.error = errorMessage;
      this.trackingStore.set(trackingId, trackingInfo);

      console.error(`❌ Failed to send email to ${to}:`, {
        error: errorMessage,
        trackingId,
        type
      });

      return {
        success: false,
        error: errorMessage
      };
    }
  }

  getEmailStatus(trackingId: string): EmailTrackingInfo | null {
    return this.trackingStore.get(trackingId) || null;
  }

  getAllEmailTracking(): EmailTrackingInfo[] {
    return Array.from(this.trackingStore.values());
  }

  // Clean up old tracking entries (older than 24 hours)
  cleanupTracking(): void {
    const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
    
    for (const [id, info] of this.trackingStore.entries()) {
      if (info.sentAt < oneDayAgo) {
        this.trackingStore.delete(id);
      }
    }
  }

  // Retry failed emails with exponential backoff
  async retryFailedEmails(): Promise<{ retried: number; stillFailed: number }> {
    const failedEmails = Array.from(this.trackingStore.values())
      .filter(info => info.status === 'failed' && info.retryCount < 3);

    let retried = 0;
    let stillFailed = 0;

    for (const emailInfo of failedEmails) {
      // Exponential backoff: wait 2^retryCount minutes
      const backoffMinutes = Math.pow(2, emailInfo.retryCount);
      const backoffTime = backoffMinutes * 60 * 1000;
      const timeSinceFailure = Date.now() - emailInfo.sentAt.getTime();

      if (timeSinceFailure >= backoffTime) {
        // Retry logic would go here - for now just increment retry count
        emailInfo.retryCount++;
        this.trackingStore.set(emailInfo.id, emailInfo);
        
        if (emailInfo.retryCount >= 3) {
          stillFailed++;
        } else {
          retried++;
        }
      }
    }

    return { retried, stillFailed };
  }
}

// Singleton instance
export const emailService = new EmailService();