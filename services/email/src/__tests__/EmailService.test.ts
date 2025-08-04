import { EmailService } from '../services/EmailService';
import { VoterInvitationData } from '../templates/voterInvitation';
import { PollResultsData } from '../templates/pollResults';

// Mock nodemailer
jest.mock('nodemailer', () => ({
  createTransport: jest.fn().mockReturnValue({
    sendMail: jest.fn().mockResolvedValue({
      messageId: 'test-message-id',
      response: '250 Message accepted'
    }),
    verify: jest.fn().mockResolvedValue(true)
  })
}));

describe('EmailService', () => {
  let emailService: EmailService;
  let mockVoterInvitationData: VoterInvitationData;
  let mockPollResultsData: PollResultsData;

  beforeEach(() => {
    emailService = new EmailService();
    
    mockVoterInvitationData = {
      pollTitle: 'Test Poll',
      pollDescription: 'A test poll for voting',
      votingUrl: 'https://rankchoice.app/vote/test-token',
      pollOwnerName: 'John Doe',
      pollOwnerEmail: 'john@example.com',
      closesAt: '2024-12-31T23:59:59Z',
      voterName: 'Jane Smith'
    };

    mockPollResultsData = {
      pollTitle: 'Test Poll',
      pollDescription: 'A test poll for voting',
      winnerName: 'Candidate A',
      totalVotes: 100,
      resultsUrl: 'https://rankchoice.app/polls/123/results',
      pollOwnerName: 'John Doe',
      voterName: 'Jane Smith',
      finalRankings: [
        { position: 1, name: 'Candidate A', votes: 60, percentage: 60 },
        { position: 2, name: 'Candidate B', votes: 40, percentage: 40 }
      ]
    };
  });

  describe('sendVoterInvitation', () => {
    it('should send voter invitation email successfully', async () => {
      const result = await emailService.sendVoterInvitation(
        mockVoterInvitationData,
        'recipient@example.com'
      );

      expect(result.success).toBe(true);
      expect(result.messageId).toBe('test-message-id');
      expect(result.error).toBeUndefined();
    });

    it('should handle email sending errors', async () => {
      // Mock a failed email send
      const nodemailer = require('nodemailer');
      nodemailer.createTransport().sendMail.mockRejectedValueOnce(
        new Error('SMTP connection failed')
      );

      const result = await emailService.sendVoterInvitation(
        mockVoterInvitationData,
        'invalid@example.com'
      );

      expect(result.success).toBe(false);
      expect(result.error).toBe('SMTP connection failed');
      expect(result.messageId).toBeUndefined();
    });
  });

  describe('sendPollResults', () => {
    it('should send poll results email successfully', async () => {
      const result = await emailService.sendPollResults(
        mockPollResultsData,
        'recipient@example.com'
      );

      expect(result.success).toBe(true);
      expect(result.messageId).toBe('test-message-id');
      expect(result.error).toBeUndefined();
    });
  });

  describe('sendBulkVoterInvitations', () => {
    it('should send bulk invitations successfully', async () => {
      const recipients = [
        'voter1@example.com',
        'voter2@example.com',
        'voter3@example.com'
      ];

      const result = await emailService.sendBulkVoterInvitations(
        mockVoterInvitationData,
        recipients
      );

      expect(result.success).toHaveLength(3);
      expect(result.failed).toHaveLength(0);
    });

    it('should handle partial failures in bulk sends', async () => {
      const nodemailer = require('nodemailer');
      const sendMail = nodemailer.createTransport().sendMail;
      
      // Mock alternating success/failure
      sendMail
        .mockResolvedValueOnce({ messageId: 'success-1' })
        .mockRejectedValueOnce(new Error('Failed'))
        .mockResolvedValueOnce({ messageId: 'success-2' });

      const recipients = [
        'voter1@example.com',
        'voter2@example.com',
        'voter3@example.com'
      ];

      const result = await emailService.sendBulkVoterInvitations(
        mockVoterInvitationData,
        recipients
      );

      expect(result.success).toHaveLength(2);
      expect(result.failed).toHaveLength(1);
    });
  });

  describe('email tracking', () => {
    it('should track email sending status', async () => {
      await emailService.sendVoterInvitation(
        mockVoterInvitationData,
        'tracked@example.com'
      );

      const allTracking = emailService.getAllEmailTracking();
      expect(allTracking).toHaveLength(1);
      
      const tracking = allTracking[0];
      expect(tracking.to).toBe('tracked@example.com');
      expect(tracking.status).toBe('sent');
      expect(tracking.messageId).toBe('test-message-id');
    });

    it('should retrieve email status by tracking ID', async () => {
      await emailService.sendVoterInvitation(
        mockVoterInvitationData,
        'tracked@example.com'
      );

      const allTracking = emailService.getAllEmailTracking();
      const trackingId = allTracking[0].id;
      
      const status = emailService.getEmailStatus(trackingId);
      expect(status).toBeDefined();
      expect(status?.to).toBe('tracked@example.com');
    });

    it('should return null for non-existent tracking ID', () => {
      const status = emailService.getEmailStatus('non-existent-id');
      expect(status).toBeNull();
    });
  });

  describe('cleanup operations', () => {
    it('should clean up old tracking entries', () => {
      // This test would need to manipulate dates to test cleanup
      // For now, just verify the method doesn't throw
      expect(() => emailService.cleanupTracking()).not.toThrow();
    });

    it('should retry failed emails', async () => {
      const result = await emailService.retryFailedEmails();
      expect(result).toHaveProperty('retried');
      expect(result).toHaveProperty('stillFailed');
      expect(typeof result.retried).toBe('number');
      expect(typeof result.stillFailed).toBe('number');
    });
  });
});