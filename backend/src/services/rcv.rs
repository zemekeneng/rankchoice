use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ballot {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub rankings: Vec<Uuid>, // Ordered list of candidate IDs (1st choice, 2nd choice, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub round_number: usize,
    pub vote_counts: HashMap<Uuid, f64>,
    pub eliminated: Option<Uuid>,
    pub winner: Option<Uuid>,
    pub exhausted_ballots: usize,
    pub total_votes: f64,
    pub majority_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcvResult {
    pub rounds: Vec<Round>,
    pub winner: Option<Uuid>,
    pub total_ballots: usize,
    pub exhausted_ballots: usize,
}

#[derive(Debug, Clone)]
pub enum TieBreakMethod {
    PreviousRounds,
    Random(u64), // seed for deterministic randomness
    BallotOrder,
}

pub struct SingleWinnerRCV {
    candidates: Vec<Candidate>,
    ballots: Vec<Ballot>,
    tie_break_method: TieBreakMethod,
}

impl SingleWinnerRCV {
    pub fn new(candidates: Vec<Candidate>, ballots: Vec<Ballot>) -> Self {
        Self {
            candidates,
            ballots,
            tie_break_method: TieBreakMethod::PreviousRounds,
        }
    }

    pub fn with_tie_break_method(mut self, method: TieBreakMethod) -> Self {
        self.tie_break_method = method;
        self
    }

    /// Validate all ballots before tabulation
    pub fn validate_ballots(&self) -> Result<(), String> {
        let candidate_ids: HashSet<Uuid> = self.candidates.iter().map(|c| c.id).collect();
        
        for ballot in &self.ballots {
            // Check for duplicate rankings
            let mut seen_candidates = HashSet::new();
            for &candidate_id in &ballot.rankings {
                if !candidate_ids.contains(&candidate_id) {
                    return Err(format!("Invalid candidate ID {} in ballot {}", candidate_id, ballot.id));
                }
                if !seen_candidates.insert(candidate_id) {
                    return Err(format!("Duplicate candidate ranking in ballot {}", ballot.id));
                }
            }
        }
        Ok(())
    }

    /// Perform RCV tabulation and return results
    pub fn tabulate(&self) -> Result<RcvResult, String> {
        // Validate ballots first
        self.validate_ballots()?;

        if self.candidates.len() < 2 {
            return Err("Need at least 2 candidates for RCV".to_string());
        }

        let mut rounds = Vec::new();
        let mut eliminated_candidates = HashSet::new();
        let mut round_number = 1;
        let total_ballots = self.ballots.len();

        loop {
            // Count votes for active candidates
            let mut vote_counts: HashMap<Uuid, f64> = HashMap::new();
            let mut exhausted_count = 0;

            for ballot in &self.ballots {
                // Find the highest-ranked non-eliminated candidate
                let vote = ballot.rankings.iter()
                    .find(|&candidate_id| !eliminated_candidates.contains(candidate_id));

                match vote {
                    Some(candidate_id) => {
                        *vote_counts.entry(*candidate_id).or_insert(0.0) += 1.0;
                    }
                    None => {
                        exhausted_count += 1;
                    }
                }
            }

            let total_votes: f64 = vote_counts.values().sum();
            let majority_threshold = total_votes / 2.0;

            // Check for winner (>50% of active votes)
            let winner = vote_counts.iter()
                .find(|(_, &count)| count > majority_threshold)
                .map(|(id, _)| *id);

            // Find candidate(s) with fewest votes for elimination
            let candidate_to_eliminate = if winner.is_none() && vote_counts.len() > 1 {
                let min_votes = vote_counts.values()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .copied()
                    .unwrap_or(0.0);

                let tied_candidates: Vec<Uuid> = vote_counts.iter()
                    .filter(|(_, &votes)| votes == min_votes)
                    .map(|(id, _)| *id)
                    .collect();

                if tied_candidates.len() == 1 {
                    Some(tied_candidates[0])
                } else {
                    // Handle tie-breaking
                    Some(self.break_tie(&tied_candidates, &rounds))
                }
            } else {
                None
            };

            // Record round results
            let round = Round {
                round_number,
                vote_counts: vote_counts.clone(),
                eliminated: candidate_to_eliminate,
                winner,
                exhausted_ballots: exhausted_count,
                total_votes,
                majority_threshold,
            };

            rounds.push(round);

            // Check termination conditions
            if winner.is_some() || vote_counts.len() <= 1 {
                break;
            }

            // Eliminate candidate
            if let Some(eliminated) = candidate_to_eliminate {
                eliminated_candidates.insert(eliminated);
            }

            round_number += 1;

            // Safety check to prevent infinite loops
            if round_number > self.candidates.len() {
                return Err("Too many rounds - possible infinite loop detected".to_string());
            }
        }

        // Determine final winner
        let final_winner = rounds.last()
            .and_then(|last_round| last_round.winner)
            .or_else(|| {
                // If no majority winner, the last remaining candidate wins
                rounds.last()
                    .and_then(|last_round| {
                        last_round.vote_counts.keys().next().copied()
                    })
            });

        let final_exhausted = rounds.last()
            .map(|r| r.exhausted_ballots)
            .unwrap_or(0);

        Ok(RcvResult {
            rounds,
            winner: final_winner,
            total_ballots,
            exhausted_ballots: final_exhausted,
        })
    }

    /// Break ties between candidates using the configured method
    fn break_tie(&self, tied_candidates: &[Uuid], previous_rounds: &[Round]) -> Uuid {
        match self.tie_break_method {
            TieBreakMethod::PreviousRounds => {
                // Look backwards through rounds for differentiation
                for round in previous_rounds.iter().rev() {
                    let mut candidate_votes: Vec<(Uuid, f64)> = tied_candidates.iter()
                        .filter_map(|&id| {
                            round.vote_counts.get(&id).map(|&votes| (id, votes))
                        })
                        .collect();
                    
                    candidate_votes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    
                    if candidate_votes.len() > 1 && 
                       candidate_votes[0].1 > candidate_votes[1].1 {
                        return candidate_votes[0].0;
                    }
                }
                
                // Fall back to first candidate if no difference found
                tied_candidates[0]
            }
            TieBreakMethod::Random(seed) => {
                use rand::{Rng, SeedableRng};
                use rand::rngs::StdRng;
                
                let mut rng = StdRng::seed_from_u64(seed);
                tied_candidates[rng.gen_range(0..tied_candidates.len())]
            }
            TieBreakMethod::BallotOrder => {
                // Return the first candidate (assumes candidates are in ballot order)
                tied_candidates[0]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candidates() -> Vec<Candidate> {
        vec![
            Candidate { id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(), name: "Alice".to_string() },
            Candidate { id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(), name: "Bob".to_string() },
            Candidate { id: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(), name: "Charlie".to_string() },
        ]
    }

    #[test]
    fn test_simple_majority_winner() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        let bob_id = candidates[1].id;
        let charlie_id = candidates[2].id;

        // 3 votes for Alice (60%), 2 votes for others
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id, charlie_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, charlie_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] },
        ];

        let rcv = SingleWinnerRCV::new(candidates, ballots);
        let result = rcv.tabulate().unwrap();

        assert_eq!(result.rounds.len(), 1);
        assert_eq!(result.winner, Some(alice_id));
        assert_eq!(result.rounds[0].vote_counts[&alice_id], 3.0);
    }

    #[test]
    fn test_rcv_elimination_and_transfer() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        let bob_id = candidates[1].id;
        let charlie_id = candidates[2].id;

        // No majority in first round, Charlie eliminated, votes transfer to Alice
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, charlie_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, charlie_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] }, // Transfers to Alice
        ];

        let rcv = SingleWinnerRCV::new(candidates, ballots);
        let result = rcv.tabulate().unwrap();

        assert_eq!(result.rounds.len(), 2);
        assert_eq!(result.rounds[0].eliminated, Some(charlie_id));
        assert_eq!(result.winner, Some(alice_id));
        
        // First round: Alice=2, Bob=2, Charlie=1
        assert_eq!(result.rounds[0].vote_counts[&alice_id], 2.0);
        assert_eq!(result.rounds[0].vote_counts[&bob_id], 2.0);
        assert_eq!(result.rounds[0].vote_counts[&charlie_id], 1.0);
        
        // Second round: Alice=3, Bob=2 (Charlie's vote transferred to Alice)
        assert_eq!(result.rounds[1].vote_counts[&alice_id], 3.0);
        assert_eq!(result.rounds[1].vote_counts[&bob_id], 2.0);
    }

    #[test]
    fn test_exhausted_ballots() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        let bob_id = candidates[1].id;
        let charlie_id = candidates[2].id;

        // Create a scenario where Charlie clearly has fewest votes and gets eliminated
        // leading to exhausted ballots
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id] }, // Will be exhausted
        ];

        let rcv = SingleWinnerRCV::new(candidates, ballots);
        let result = rcv.tabulate().unwrap();

        // Charlie gets eliminated, then Alice vs Bob tie broken by previous rounds
        assert!(result.rounds.len() >= 2);
        assert_eq!(result.rounds[0].eliminated, Some(charlie_id));
        
        // Final round should have 1 exhausted ballot
        let final_round = result.rounds.last().unwrap();
        assert_eq!(final_round.exhausted_ballots, 1);
        
        // First round vote counts: Alice=2, Bob=2, Charlie=1
        // Charlie should be eliminated (has clearly fewest votes)
        assert_eq!(result.rounds[0].vote_counts[&alice_id], 2.0);
        assert_eq!(result.rounds[0].vote_counts[&bob_id], 2.0);
        assert_eq!(result.rounds[0].vote_counts[&charlie_id], 1.0);
    }

    #[test]
    fn test_invalid_ballot_validation() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        
        // Ballot with duplicate candidate
        let ballots = vec![
            Ballot { 
                id: Uuid::new_v4(), 
                voter_id: Uuid::new_v4(), 
                rankings: vec![alice_id, alice_id] // Duplicate!
            },
        ];

        let rcv = SingleWinnerRCV::new(candidates, ballots);
        let result = rcv.tabulate();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate candidate"));
    }

    #[test]
    fn test_tie_breaking_previous_rounds() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        let bob_id = candidates[1].id;
        let charlie_id = candidates[2].id;

        // Create a scenario with a clear winner after eliminations
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, bob_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, charlie_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] },
        ];

        let rcv = SingleWinnerRCV::new(candidates, ballots)
            .with_tie_break_method(TieBreakMethod::PreviousRounds);
        let result = rcv.tabulate().unwrap();

        // Alice should win with majority after transfers
        assert_eq!(result.winner, Some(alice_id));
        
        // Should have multiple rounds due to eliminations
        assert!(result.rounds.len() >= 2);
    }
} 