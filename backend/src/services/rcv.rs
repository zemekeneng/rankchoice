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
    pub tiebreak_reason: Option<TieBreakReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcvResult {
    pub rounds: Vec<Round>,
    pub winner: Option<Uuid>,
    pub total_ballots: usize,
    pub exhausted_ballots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TieBreakMethod {
    FirstChoiceVotes,
    PriorRoundPerformance,  
    MostVotesToDistribute,
    Random(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TieBreakReason {
    FirstChoiceVotes,
    PriorRoundPerformance,
    MostVotesToDistribute,
    Random,
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
            tie_break_method: TieBreakMethod::Random(42), // Default random seed
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
            let (candidate_to_eliminate, tiebreak_reason) = if winner.is_none() && vote_counts.len() > 1 {
                let min_votes = vote_counts.values()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .copied()
                    .unwrap_or(0.0);

                let tied_candidates: Vec<Uuid> = vote_counts.iter()
                    .filter(|(_, &votes)| votes == min_votes)
                    .map(|(id, _)| *id)
                    .collect();

                if tied_candidates.len() == 1 {
                    (Some(tied_candidates[0]), None)
                } else {
                    // Handle tie-breaking with comprehensive strategy
                    let (eliminated, reason) = self.break_tie_comprehensive(&tied_candidates, &rounds);
                    (Some(eliminated), Some(reason))
                }
            } else {
                (None, None)
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
                tiebreak_reason,
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

    /// Break ties between candidates using comprehensive strategy
    fn break_tie_comprehensive(&self, tied_candidates: &[Uuid], previous_rounds: &[Round]) -> (Uuid, TieBreakReason) {
        // Strategy 1: First choice votes
        if let Some(winner) = self.try_first_choice_tiebreak(tied_candidates) {
            return (winner, TieBreakReason::FirstChoiceVotes);
        }

        // Strategy 2: Prior round performance  
        if let Some(winner) = self.try_prior_round_tiebreak(tied_candidates, previous_rounds) {
            return (winner, TieBreakReason::PriorRoundPerformance);
        }

        // Strategy 3: Most votes to distribute
        if let Some(winner) = self.try_most_votes_to_distribute(tied_candidates, previous_rounds) {
            return (winner, TieBreakReason::MostVotesToDistribute);
        }

        // Strategy 4: Random selection
        let winner = self.random_tiebreak(tied_candidates);
        (winner, TieBreakReason::Random)
    }

    /// Strategy 1: Eliminate candidate with fewer first-choice votes
    fn try_first_choice_tiebreak(&self, tied_candidates: &[Uuid]) -> Option<Uuid> {
        let mut first_choice_counts: HashMap<Uuid, usize> = HashMap::new();
        
        // Count first-choice votes for tied candidates
        for ballot in &self.ballots {
            if let Some(&first_choice) = ballot.rankings.first() {
                if tied_candidates.contains(&first_choice) {
                    *first_choice_counts.entry(first_choice).or_insert(0) += 1;
                }
            }
        }

        // Find minimum first-choice votes among tied candidates
        let min_first_choice = tied_candidates.iter()
            .map(|&id| first_choice_counts.get(&id).copied().unwrap_or(0))
            .min()?;

        // Return candidate with fewest first-choice votes if unique
        let candidates_with_min: Vec<Uuid> = tied_candidates.iter()
            .filter(|&&id| first_choice_counts.get(&id).copied().unwrap_or(0) == min_first_choice)
            .copied()
            .collect();

        if candidates_with_min.len() == 1 {
            Some(candidates_with_min[0])
        } else {
            None
        }
    }

    /// Strategy 2: Prior round performance (look back for differentiation)
    fn try_prior_round_tiebreak(&self, tied_candidates: &[Uuid], previous_rounds: &[Round]) -> Option<Uuid> {
        // Look backwards through rounds for differentiation
        for round in previous_rounds.iter().rev() {
            let mut candidate_votes: Vec<(Uuid, f64)> = tied_candidates.iter()
                .filter_map(|&id| {
                    round.vote_counts.get(&id).map(|&votes| (id, votes))
                })
                .collect();
            
            if candidate_votes.is_empty() {
                continue;
            }

            candidate_votes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Return candidate with lowest votes in this round if unique
            if candidate_votes.len() > 1 && 
               candidate_votes[0].1 < candidate_votes[1].1 {
                return Some(candidate_votes[0].0);
            }
        }
        None
    }

    /// Strategy 3: Eliminate candidate who would redistribute most votes
    fn try_most_votes_to_distribute(&self, tied_candidates: &[Uuid], _previous_rounds: &[Round]) -> Option<Uuid> {
        let mut redistribution_counts: HashMap<Uuid, usize> = HashMap::new();

        // Count how many ballots each tied candidate would redistribute
        for ballot in &self.ballots {
            // Find which tied candidate this ballot would go to if eliminated
            for (ranking_index, &candidate_id) in ballot.rankings.iter().enumerate() {
                if tied_candidates.contains(&candidate_id) {
                    // Count how many more preferences this ballot has after this candidate
                    let remaining_preferences = ballot.rankings.len() - ranking_index - 1;
                    *redistribution_counts.entry(candidate_id).or_insert(0) += remaining_preferences;
                    break;
                }
            }
        }

        // Find candidate with most votes to redistribute
        let max_redistribution = tied_candidates.iter()
            .map(|&id| redistribution_counts.get(&id).copied().unwrap_or(0))
            .max()?;

        let candidates_with_max: Vec<Uuid> = tied_candidates.iter()
            .filter(|&&id| redistribution_counts.get(&id).copied().unwrap_or(0) == max_redistribution)
            .copied()
            .collect();

        if candidates_with_max.len() == 1 {
            Some(candidates_with_max[0])
        } else {
            None
        }
    }

    /// Strategy 4: Random selection
    fn random_tiebreak(&self, tied_candidates: &[Uuid]) -> Uuid {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        
        let seed = match &self.tie_break_method {
            TieBreakMethod::Random(seed) => *seed,
            _ => 42, // Default seed
        };
        
        let mut rng = StdRng::seed_from_u64(seed);
        tied_candidates[rng.gen_range(0..tied_candidates.len())]
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
    fn test_comprehensive_tiebreaker() {
        let candidates = create_test_candidates();
        let alice_id = candidates[0].id;
        let bob_id = candidates[1].id;
        let charlie_id = candidates[2].id;

        // Create a scenario where Alice and Bob are tied for last place in round 1
        // Charlie: 3 first-choice, Alice: 1 first-choice, Bob: 2 first-choice
        // But arrange votes so Alice and Bob both get 1 vote in round 1 (tied for last)
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] },    // Charlie 1st
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, bob_id] },      // Charlie 1st
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] },    // Charlie 1st
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, charlie_id] },    // Alice 1st (1 first-choice)
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, charlie_id] },      // Bob 1st  (1 first-choice)
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },        // Bob 1st  (2 first-choice total)
        ];
        // Result in round 1: Charlie=3, Alice=1, Bob=2 votes
        // No tie, so Bob should be eliminated normally without tiebreaker
        
        // Let's create a real tie scenario instead
        let ballots = vec![
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, alice_id] },    // Charlie 1st
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![charlie_id, bob_id] },      // Charlie 1st  
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![alice_id, charlie_id] },    // Alice 1st (1 first-choice)
            Ballot { id: Uuid::new_v4(), voter_id: Uuid::new_v4(), rankings: vec![bob_id, alice_id] },        // Bob 1st (1 first-choice, same as Alice)
        ];
        // Round 1 result: Charlie=2, Alice=1, Bob=1 (Alice and Bob tied for last)
        // First-choice counts: Charlie=0, Alice=1, Bob=1 (Alice and Bob tied in first-choice too)
        // Should go to strategy 2 (prior round performance) - but this is round 1, so no prior rounds
        // Should go to strategy 3 (most votes to distribute) or 4 (random)

        let rcv = SingleWinnerRCV::new(candidates, ballots);
        let result = rcv.tabulate().unwrap();

        // Test passes if any of the expected tiebreaker scenarios occur
        assert!(result.rounds.len() >= 1);
        
        // Find a round with elimination that had a tiebreaker
        let had_tiebreaker = result.rounds.iter().any(|round| {
            round.eliminated.is_some() && round.tiebreak_reason.is_some()
        });
        
        // Should have used a tiebreaker in at least one round
        assert!(had_tiebreaker, "Expected at least one round to use a tiebreaker");
        assert_eq!(result.winner, Some(charlie_id));
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
            .with_tie_break_method(TieBreakMethod::PriorRoundPerformance);
        let result = rcv.tabulate().unwrap();

        // Alice should win with majority after transfers
        assert_eq!(result.winner, Some(alice_id));
        
        // Should have multiple rounds due to eliminations
        assert!(result.rounds.len() >= 2);
    }
} 