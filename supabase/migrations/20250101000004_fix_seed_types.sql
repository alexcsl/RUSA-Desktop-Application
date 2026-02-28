-- Fix test request types to match Rust query expectations
-- Financial: type should be 'budget' not 'financial'
-- Data requests: type should be 'data' with bypass_authority set

UPDATE requests SET type = 'budget' WHERE type = 'financial';
UPDATE requests SET type = 'data', bypass_authority = 'TheStatistician' WHERE type = 'data_request';
