# Ranked-Choice Voting Algorithm Implementation

## Overview

This document details the implementation of the Ranked-Choice Voting (RCV) algorithm for RankChoice.app, covering both single-winner and multi-winner elections.

## Single-Winner RCV Algorithm

### Process Overview

1. **First Choice Count**: Count all first-choice votes
2. **Majority Check**: If a candidate has >50% of votes, they win
3. **Elimination**: If no majority, eliminate the candidate with fewest votes
4. **Redistribution**: Transfer eliminated candidate's votes to voters' next choices
5. **Repeat**: Continue until a candidate achieves majority

### Rust Implementation

```rust
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Ballot {
    pub id: Uuid,
    pub rankings: Vec<Uuid>, // Ordered list of candidate IDs
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct Round {
    pub round_number: usize,
    pub vote_counts: HashMap<Uuid, f64>,
    pub eliminated: Option<Uuid>,
    pub winner: Option<Uuid>,
    pub exhausted_ballots: usize,
}

pub struct SingleWinnerRCV {
    candidates: Vec<Candidate>,
    ballots: Vec<Ballot>,
}

impl SingleWinnerRCV {
    pub fn new(candidates: Vec<Candidate>, ballots: Vec<Ballot>) -> Self {
        Self { candidates, ballots }
    }

    pub fn tabulate(&self) -> Vec<Round> {
        let mut rounds = Vec::new();
        let mut active_ballots = self.ballots.clone();
        let mut eliminated_candidates = HashSet::new();
        let mut round_number = 1;

        loop {
            // Count votes for active candidates
            let mut vote_counts: HashMap<Uuid, f64> = HashMap::new();
            let mut exhausted_count = 0;

            for ballot in &active_ballots {
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

            // Check for winner
            let winner = vote_counts.iter()
                .find(|(_, &count)| count > majority_threshold)
                .map(|(id, _)| *id);

            // Find candidate with fewest votes
            let candidate_to_eliminate = if winner.is_none() && vote_counts.len() > 1 {
                vote_counts.iter()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(id, _)| *id)
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
        }

        rounds
    }
}
```

## Multi-Winner RCV Algorithm (Single Transferable Vote)

### Process Overview

1. **Calculate Threshold**: Determine the minimum votes needed to win (Droop quota)
2. **First Choice Count**: Count all first-choice votes
3. **Elect Winners**: Any candidate exceeding the threshold is elected
4. **Transfer Surplus**: Redistribute excess votes proportionally
5. **Eliminate Lowest**: If seats remain, eliminate the lowest candidate
6. **Repeat**: Continue until all seats are filled

### Droop Quota Calculation

```
Threshold = (Total Valid Votes / (Seats + 1)) + 1
```

### Rust Implementation

```rust
pub struct MultiWinnerRCV {
    candidates: Vec<Candidate>,
    ballots: Vec<Ballot>,
    num_winners: usize,
}

#[derive(Debug)]
pub struct MultiWinnerRound {
    pub round_number: usize,
    pub vote_counts: HashMap<Uuid, f64>,
    pub elected: Vec<Uuid>,
    pub eliminated: Option<Uuid>,
    pub exhausted_ballots: usize,
    pub threshold: f64,
    pub transfer_values: HashMap<Uuid, f64>,
}

impl MultiWinnerRCV {
    pub fn new(candidates: Vec<Candidate>, ballots: Vec<Ballot>, num_winners: usize) -> Self {
        Self {
            candidates,
            ballots,
            num_winners,
        }
    }

    pub fn tabulate(&self) -> Vec<MultiWinnerRound> {
        let mut rounds = Vec::new();
        let mut elected_candidates = HashSet::new();
        let mut eliminated_candidates = HashSet::new();
        let mut ballot_weights: HashMap<Uuid, f64> = self.ballots.iter()
            .map(|b| (b.id, 1.0))
            .collect();
        let mut round_number = 1;

        // Calculate Droop quota
        let total_votes = self.ballots.len() as f64;
        let threshold = (total_votes / (self.num_winners + 1) as f64).floor() + 1.0;

        while elected_candidates.len() < self.num_winners {
            // Count weighted votes
            let mut vote_counts: HashMap<Uuid, f64> = HashMap::new();
            let mut exhausted_count = 0.0;

            for ballot in &self.ballots {
                let weight = ballot_weights.get(&ballot.id).unwrap_or(&0.0);
                if *weight == 0.0 {
                    continue;
                }

                let vote = ballot.rankings.iter()
                    .find(|&candidate_id| {
                        !elected_candidates.contains(candidate_id) &&
                        !eliminated_candidates.contains(candidate_id)
                    });

                match vote {
                    Some(candidate_id) => {
                        *vote_counts.entry(*candidate_id).or_insert(0.0) += weight;
                    }
                    None => {
                        exhausted_count += weight;
                    }
                }
            }

            // Check for candidates meeting threshold
            let mut newly_elected = Vec::new();
            let mut transfer_values = HashMap::new();

            for (candidate_id, &vote_count) in &vote_counts {
                if vote_count >= threshold {
                    newly_elected.push(*candidate_id);
                    elected_candidates.insert(*candidate_id);
                    
                    // Calculate transfer value for surplus
                    let surplus = vote_count - threshold;
                    let transfer_value = surplus / vote_count;
                    transfer_values.insert(*candidate_id, transfer_value);
                }
            }

            // If no one elected this round and seats remain, eliminate lowest
            let eliminated = if newly_elected.is_empty() && 
                elected_candidates.len() < self.num_winners &&
                vote_counts.len() > (self.num_winners - elected_candidates.len()) {
                
                vote_counts.iter()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(id, _)| {
                        eliminated_candidates.insert(*id);
                        *id
                    })
            } else {
                None
            };

            // Update ballot weights for elected candidates
            for elected_id in &newly_elected {
                let transfer_value = transfer_values.get(elected_id).unwrap_or(&0.0);
                
                for ballot in &self.ballots {
                    if ballot.rankings.iter()
                        .find(|&&id| !elected_candidates.contains(&id) && 
                                     !eliminated_candidates.contains(&id))
                        .map(|&id| id == *elected_id)
                        .unwrap_or(false) {
                        
                        let current_weight = ballot_weights.get(&ballot.id).unwrap_or(&1.0);
                        ballot_weights.insert(ballot.id, current_weight * transfer_value);
                    }
                }
            }

            // Record round
            rounds.push(MultiWinnerRound {
                round_number,
                vote_counts,
                elected: newly_elected,
                eliminated,
                exhausted_ballots: exhausted_count as usize,
                threshold,
                transfer_values,
            });

            round_number += 1;

            // Stop if we've elected enough or can't elect more
            if elected_candidates.len() >= self.num_winners ||
               vote_counts.is_empty() {
                break;
            }
        }

        rounds
    }
}
```

## Tie-Breaking Strategies

When candidates have equal votes, we need deterministic tie-breaking:

1. **Previous Round Performance**: Compare votes in earlier rounds
2. **Random Selection**: Use a seeded random number generator
3. **Candidate Order**: Use ballot order (least preferred)

```rust
pub enum TieBreakMethod {
    PreviousRounds,
    Random(u64), // seed
    BallotOrder,
}

fn break_tie(
    candidates: &[Uuid],
    method: &TieBreakMethod,
    previous_rounds: &[Round],
) -> Uuid {
    match method {
        TieBreakMethod::PreviousRounds => {
            // Look backwards through rounds for differentiation
            for round in previous_rounds.iter().rev() {
                let mut candidate_votes: Vec<(Uuid, f64)> = candidates.iter()
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
            
            // Fall back to random if no difference found
            candidates[0]
        }
        TieBreakMethod::Random(seed) => {
            use rand::{Rng, SeedableRng};
            use rand::rngs::StdRng;
            
            let mut rng = StdRng::seed_from_u64(*seed);
            candidates[rng.gen_range(0..candidates.len())]
        }
        TieBreakMethod::BallotOrder => {
            candidates[0] // Assumes candidates are in ballot order
        }
    }
}
```

## Validation Rules

### Ballot Validation
- No duplicate rankings
- Rankings must be sequential (1, 2, 3, not 1, 3, 5)
- Only valid candidate IDs
- Maximum rankings enforced

### Election Validation
- Minimum 2 candidates
- Number of winners < number of candidates
- All ballots validated before tabulation

## Performance Considerations

1. **Memory Efficiency**: Use references where possible
2. **Parallel Processing**: Count ballots in parallel for large elections
3. **Caching**: Cache intermediate results for real-time updates
4. **Database Optimization**: Store pre-computed results for common queries

## Testing Strategy

1. **Unit Tests**: Test each component in isolation
2. **Property-Based Testing**: Use proptest for edge cases
3. **Benchmark Tests**: Ensure performance at scale
4. **Real-World Data**: Test with actual election data

## References

- [FairVote RCV Resources](https://www.fairvote.org/rcv)
- [Single Transferable Vote Foundation](https://www.electoral-reform.org.uk/voting-systems/types-of-voting-system/single-transferable-vote/)
- [Portland RCV Implementation](https://www.portland.gov/vote/ranked-choice-voting) 