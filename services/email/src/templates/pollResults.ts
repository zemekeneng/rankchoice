import { EmailTemplate } from '../config/email';

export interface PollResultsData {
  pollTitle: string;
  pollDescription?: string;
  winnerName: string;
  totalVotes: number;
  resultsUrl: string;
  pollOwnerName: string;
  voterName?: string;
  finalRankings: Array<{
    position: number;
    name: string;
    votes: number;
    percentage: number;
  }>;
}

export function createPollResultsTemplate(data: PollResultsData): EmailTemplate {
  const voterGreeting = data.voterName ? `Hi ${data.voterName}` : 'Hello';
  
  const subject = `Poll Results: ${data.winnerName} wins "${data.pollTitle}"`;

  const text = `
${voterGreeting},

The results are in for "${data.pollTitle}"!

🏆 Winner: ${data.winnerName}
📊 Total votes: ${data.totalVotes}

Final Rankings:
${data.finalRankings.map(r => 
  `${r.position}. ${r.name} - ${r.votes} votes (${r.percentage.toFixed(1)}%)`
).join('\n')}

View detailed results and see the round-by-round voting process:
${data.resultsUrl}

Thank you for participating in this ranked choice vote organized by ${data.pollOwnerName}!

The RankChoice.app Team
  `.trim();

  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Poll Results</title>
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
        .winner-announcement {
            background: linear-gradient(135deg, #10b981, #059669);
            color: white;
            padding: 24px;
            border-radius: 12px;
            text-align: center;
            margin: 20px 0;
        }
        .winner-name {
            font-size: 24px;
            font-weight: bold;
            margin: 10px 0;
        }
        .poll-title {
            font-size: 18px;
            margin-bottom: 5px;
        }
        .vote-count {
            font-size: 16px;
            opacity: 0.9;
        }
        .rankings {
            margin: 30px 0;
        }
        .ranking-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 16px;
            margin: 8px 0;
            border-radius: 8px;
            background-color: #f9fafb;
            border-left: 4px solid #e5e7eb;
        }
        .ranking-item.winner {
            background-color: #ecfdf5;
            border-left-color: #10b981;
        }
        .ranking-position {
            font-weight: bold;
            font-size: 18px;
            color: #4b5563;
            margin-right: 12px;
        }
        .ranking-name {
            flex-grow: 1;
            font-weight: 500;
        }
        .ranking-votes {
            text-align: right;
            color: #6b7280;
        }
        .results-button {
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
            <div class="logo">🗳️ RankChoice.app</div>
            <h1>Poll Results Are In!</h1>
        </div>

        <p>${voterGreeting},</p>
        
        <p>The voting has concluded for "${data.pollTitle}"!</p>

        <div class="winner-announcement">
            <div>🏆 <strong>WINNER</strong></div>
            <div class="winner-name">${data.winnerName}</div>
            <div class="poll-title">"${data.pollTitle}"</div>
            <div class="vote-count">📊 ${data.totalVotes} total votes cast</div>
        </div>

        <div class="rankings">
            <h3>Final Rankings:</h3>
            ${data.finalRankings.map((ranking, index) => `
                <div class="ranking-item ${index === 0 ? 'winner' : ''}">
                    <div class="ranking-position">${ranking.position}</div>
                    <div class="ranking-name">${ranking.name}</div>
                    <div class="ranking-votes">
                        <div>${ranking.votes} votes</div>
                        <div style="font-size: 14px;">${ranking.percentage.toFixed(1)}%</div>
                    </div>
                </div>
            `).join('')}
        </div>

        <div style="text-align: center;">
            <a href="${data.resultsUrl}" class="results-button">📊 View Detailed Results</a>
        </div>

        <p>See the complete round-by-round ranked choice voting process and how the winner was determined!</p>

        <div class="footer">
            <p>Thank you for participating in this vote organized by <strong>${data.pollOwnerName}</strong></p>
            <p>Powered by RankChoice.app - Democratic voting made simple</p>
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