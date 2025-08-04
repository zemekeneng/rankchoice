import { EmailTemplate } from '../config/email';

export interface VoterInvitationData {
  pollTitle: string;
  pollDescription?: string;
  votingUrl: string;
  pollOwnerName: string;
  pollOwnerEmail: string;
  closesAt?: string;
  voterName?: string;
}

export function createVoterInvitationTemplate(data: VoterInvitationData): EmailTemplate {
  const voterGreeting = data.voterName ? `Hi ${data.voterName}` : 'Hello';
  const closingInfo = data.closesAt 
    ? `This poll closes on ${new Date(data.closesAt).toLocaleString()}.`
    : 'Please vote when you have a moment.';

  const subject = `You're invited to vote: ${data.pollTitle}`;

  const text = `
${voterGreeting},

You've been invited to participate in a ranked choice vote by ${data.pollOwnerName}.

Poll: ${data.pollTitle}
${data.pollDescription ? `Description: ${data.pollDescription}` : ''}

To vote, click this link or copy it into your browser:
${data.votingUrl}

${closingInfo}

This is a ranked choice voting poll - you can rank the candidates in order of your preference. Your backup choices will be used if your first choice doesn't have enough support.

If you have any questions, you can contact the poll organizer at ${data.pollOwnerEmail}.

Happy voting!
The RankChoice.app Team
  `.trim();

  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Voting Invitation</title>
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
        .poll-title {
            font-size: 20px;
            font-weight: 600;
            color: #1f2937;
            margin: 20px 0 10px 0;
        }
        .poll-description {
            color: #6b7280;
            margin-bottom: 20px;
        }
        .vote-button {
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
        .vote-button:hover {
            background-color: #4338ca;
        }
        .info-box {
            background-color: #f3f4f6;
            padding: 16px;
            border-radius: 8px;
            margin: 20px 0;
        }
        .closing-info {
            color: #d97706;
            font-weight: 500;
        }
        .footer {
            text-align: center;
            margin-top: 30px;
            color: #6b7280;
            font-size: 14px;
            border-top: 1px solid #e5e7eb;
            padding-top: 20px;
        }
        .voting-explanation {
            background-color: #ecfdf5;
            border-left: 4px solid #10b981;
            padding: 16px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🗳️ RankChoice.app</div>
            <h1>You're Invited to Vote!</h1>
        </div>

        <p>${voterGreeting},</p>
        
        <p>You've been invited to participate in a ranked choice vote by <strong>${data.pollOwnerName}</strong>.</p>

        <div class="poll-title">${data.pollTitle}</div>
        ${data.pollDescription ? `<div class="poll-description">${data.pollDescription}</div>` : ''}

        <div style="text-align: center;">
            <a href="${data.votingUrl}" class="vote-button">🗳️ Cast Your Vote</a>
        </div>

        <div class="voting-explanation">
            <strong>📊 How Ranked Choice Voting Works:</strong><br>
            Rank the candidates in order of your preference. If your first choice doesn't have enough support, your vote will transfer to your second choice, and so on. This ensures your voice is heard even if your top pick doesn't win!
        </div>

        ${data.closesAt ? `
        <div class="info-box">
            <div class="closing-info">⏰ Poll closes: ${new Date(data.closesAt).toLocaleString()}</div>
        </div>
        ` : ''}

        <div class="info-box">
            <strong>Need help?</strong><br>
            Contact the poll organizer: <a href="mailto:${data.pollOwnerEmail}">${data.pollOwnerEmail}</a>
        </div>

        <div class="footer">
            <p>This invitation was sent via RankChoice.app</p>
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