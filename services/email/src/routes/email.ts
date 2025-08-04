import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { emailService } from '../services/EmailService';
import { VoterInvitationData } from '../templates/voterInvitation';
import { PollResultsData } from '../templates/pollResults';

const router = Router();

// Validation schemas
const VoterInvitationSchema = z.object({
  pollTitle: z.string().min(1),
  pollDescription: z.string().optional(),
  votingUrl: z.string().url(),
  pollOwnerName: z.string().min(1),
  pollOwnerEmail: z.string().email(),
  closesAt: z.string().optional(),
  voterName: z.string().optional(),
  to: z.string().email()
});

const BulkVoterInvitationSchema = z.object({
  pollTitle: z.string().min(1),
  pollDescription: z.string().optional(),
  votingUrl: z.string().url(),
  pollOwnerName: z.string().min(1),
  pollOwnerEmail: z.string().email(),
  closesAt: z.string().optional(),
  recipients: z.array(z.object({
    email: z.string().email(),
    name: z.string().optional()
  })).min(1).max(100) // Limit bulk sends to 100 at a time
});

const PollResultsSchema = z.object({
  pollTitle: z.string().min(1),
  pollDescription: z.string().optional(),
  winnerName: z.string().min(1),
  totalVotes: z.number().min(0),
  resultsUrl: z.string().url(),
  pollOwnerName: z.string().min(1),
  voterName: z.string().optional(),
  finalRankings: z.array(z.object({
    position: z.number().min(1),
    name: z.string().min(1),
    votes: z.number().min(0),
    percentage: z.number().min(0).max(100)
  })),
  to: z.string().email()
});

// POST /api/email/voter-invitation - Send single voter invitation
router.post('/voter-invitation', async (req: Request, res: Response) => {
  try {
    const validated = VoterInvitationSchema.parse(req.body);
    
    const { to, ...invitationData } = validated;
    
    const result = await emailService.sendVoterInvitation(invitationData as VoterInvitationData, to);
    
    if (result.success) {
      res.json({
        success: true,
        data: {
          messageId: result.messageId,
          recipient: to
        }
      });
    } else {
      res.status(500).json({
        success: false,
        error: {
          code: 'EMAIL_SEND_FAILED',
          message: result.error || 'Failed to send email'
        }
      });
    }
  } catch (error) {
    if (error instanceof z.ZodError) {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request data',
          details: error.errors
        }
      });
    } else {
      console.error('Error sending voter invitation:', error);
      res.status(500).json({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Internal server error'
        }
      });
    }
  }
});

// POST /api/email/bulk-voter-invitations - Send bulk voter invitations
router.post('/bulk-voter-invitations', async (req: Request, res: Response) => {
  try {
    const validated = BulkVoterInvitationSchema.parse(req.body);
    
    const { recipients, ...baseData } = validated;
    
    // Send emails in batches to avoid overwhelming the email service
    const batchSize = 10;
    const batches = [];
    
    for (let i = 0; i < recipients.length; i += batchSize) {
      batches.push(recipients.slice(i, i + batchSize));
    }
    
    let totalSent = 0;
    let totalFailed = 0;
    const failedRecipients: string[] = [];
    
    for (const batch of batches) {
      const batchEmails = batch.map(r => r.email);
      const results = await emailService.sendBulkVoterInvitations(baseData as VoterInvitationData, batchEmails);
      
      totalSent += results.success.length;
      totalFailed += results.failed.length;
      
      // Track failed recipients
      results.failed.forEach((_, index) => {
        failedRecipients.push(batch[index].email);
      });
      
      // Small delay between batches to be respectful
      if (batches.length > 1) {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    
    res.json({
      success: true,
      data: {
        totalRecipients: recipients.length,
        sent: totalSent,
        failed: totalFailed,
        failedRecipients: failedRecipients.length > 0 ? failedRecipients : undefined
      }
    });
    
  } catch (error) {
    if (error instanceof z.ZodError) {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request data',
          details: error.errors
        }
      });
    } else {
      console.error('Error sending bulk voter invitations:', error);
      res.status(500).json({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Internal server error'
        }
      });
    }
  }
});

// POST /api/email/poll-results - Send poll results notification
router.post('/poll-results', async (req: Request, res: Response) => {
  try {
    const validated = PollResultsSchema.parse(req.body);
    
    const { to, ...resultsData } = validated;
    
    const result = await emailService.sendPollResults(resultsData as PollResultsData, to);
    
    if (result.success) {
      res.json({
        success: true,
        data: {
          messageId: result.messageId,
          recipient: to
        }
      });
    } else {
      res.status(500).json({
        success: false,
        error: {
          code: 'EMAIL_SEND_FAILED',
          message: result.error || 'Failed to send email'
        }
      });
    }
  } catch (error) {
    if (error instanceof z.ZodError) {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request data',
          details: error.errors
        }
      });
    } else {
      console.error('Error sending poll results:', error);
      res.status(500).json({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Internal server error'
        }
      });
    }
  }
});

// GET /api/email/status/:trackingId - Get email delivery status
router.get('/status/:trackingId', (req: Request, res: Response) => {
  const { trackingId } = req.params;
  
  const status = emailService.getEmailStatus(trackingId);
  
  if (status) {
    res.json({
      success: true,
      data: status
    });
  } else {
    res.status(404).json({
      success: false,
      error: {
        code: 'NOT_FOUND',
        message: 'Email tracking ID not found'
      }
    });
  }
});

// GET /api/email/tracking - Get all email tracking info (admin endpoint)
router.get('/tracking', (req: Request, res: Response) => {
  const allTracking = emailService.getAllEmailTracking();
  
  res.json({
    success: true,
    data: {
      total: allTracking.length,
      tracking: allTracking
    }
  });
});

// POST /api/email/retry-failed - Retry failed emails
router.post('/retry-failed', async (req: Request, res: Response) => {
  try {
    const result = await emailService.retryFailedEmails();
    
    res.json({
      success: true,
      data: result
    });
  } catch (error) {
    console.error('Error retrying failed emails:', error);
    res.status(500).json({
      success: false,
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to retry emails'
      }
    });
  }
});

// POST /api/email/cleanup - Clean up old tracking data
router.post('/cleanup', (req: Request, res: Response) => {
  try {
    emailService.cleanupTracking();
    
    res.json({
      success: true,
      data: {
        message: 'Tracking data cleaned up successfully'
      }
    });
  } catch (error) {
    console.error('Error cleaning up tracking data:', error);
    res.status(500).json({
      success: false,
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to cleanup tracking data'
      }
    });
  }
});

export { router as emailRouter };