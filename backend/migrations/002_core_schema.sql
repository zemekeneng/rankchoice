-- Polls table
CREATE TABLE polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    poll_type VARCHAR(50) NOT NULL DEFAULT 'single_winner',
    num_winners INTEGER DEFAULT 1,
    opens_at TIMESTAMP WITH TIME ZONE,
    closes_at TIMESTAMP WITH TIME ZONE,
    is_public BOOLEAN DEFAULT false,
    registration_required BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Candidates/Choices table
CREATE TABLE candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    display_order INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Voters table
CREATE TABLE voters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    email VARCHAR(255),
    ballot_token VARCHAR(255) UNIQUE NOT NULL,
    ip_address INET,
    user_agent TEXT,
    location_data JSONB,
    demographics JSONB,
    invited_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    voted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(poll_id, email)
);

-- Ballots table
CREATE TABLE ballots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voter_id UUID REFERENCES voters(id) ON DELETE CASCADE,
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ip_address INET,
    UNIQUE(voter_id)
);

-- Rankings table
CREATE TABLE rankings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ballot_id UUID REFERENCES ballots(id) ON DELETE CASCADE,
    candidate_id UUID REFERENCES candidates(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    UNIQUE(ballot_id, candidate_id),
    UNIQUE(ballot_id, rank)
);

-- Advertisements table (for future monetization features)
CREATE TABLE advertisements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    content_url TEXT NOT NULL,
    link_url TEXT,
    target_demographics JSONB,
    target_locations JSONB,
    weight INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Ad impressions table
CREATE TABLE ad_impressions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ad_id UUID REFERENCES advertisements(id),
    voter_id UUID REFERENCES voters(id),
    displayed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    clicked BOOLEAN DEFAULT false
);

-- Create indexes for performance
CREATE INDEX idx_polls_user_id ON polls(user_id);
CREATE INDEX idx_polls_status ON polls(opens_at, closes_at, is_public);
CREATE INDEX idx_candidates_poll_id ON candidates(poll_id);
CREATE INDEX idx_candidates_display_order ON candidates(poll_id, display_order);
CREATE INDEX idx_voters_poll_id ON voters(poll_id);
CREATE INDEX idx_voters_ballot_token ON voters(ballot_token);
CREATE INDEX idx_ballots_poll_id ON ballots(poll_id);
CREATE INDEX idx_ballots_voter_id ON ballots(voter_id);
CREATE INDEX idx_rankings_ballot_id ON rankings(ballot_id);
CREATE INDEX idx_rankings_candidate_id ON rankings(candidate_id);
CREATE INDEX idx_ad_impressions_ad_id ON ad_impressions(ad_id);
CREATE INDEX idx_ad_impressions_voter_id ON ad_impressions(voter_id);

-- Add updated_at triggers for polls table
CREATE TRIGGER update_polls_updated_at BEFORE UPDATE ON polls
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add constraints for poll validation
ALTER TABLE polls ADD CONSTRAINT polls_num_winners_positive CHECK (num_winners > 0);
ALTER TABLE polls ADD CONSTRAINT polls_valid_type CHECK (poll_type IN ('single_winner', 'multi_winner'));
ALTER TABLE polls ADD CONSTRAINT polls_valid_dates CHECK (opens_at IS NULL OR closes_at IS NULL OR opens_at < closes_at);

-- Add constraints for rankings validation
ALTER TABLE rankings ADD CONSTRAINT rankings_rank_positive CHECK (rank > 0); 