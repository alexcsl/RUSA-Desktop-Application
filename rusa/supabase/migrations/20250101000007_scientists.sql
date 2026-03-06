-- ============================================================
-- 007_scientists.sql
-- Subsystem 05 — Scientists (experiments, JSONB archives, math results)
-- Source of truth: 04_SCIENTISTS.md + 01_ENGINEERS.md (shared tables)
-- ============================================================

-- ============================================================
-- APPROVED TESTS  (shared by Engineers + Scientists)
-- Filtered by applicable_scope per role
-- ============================================================
CREATE TABLE approved_tests (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name             TEXT NOT NULL,
  procedure        TEXT NOT NULL,
  category         TEXT NOT NULL,
  -- applicable_scope drives which roles can see/use this test
  applicable_scope TEXT NOT NULL CHECK (applicable_scope IN ('plants', 'all_species', 'matter', 'physical')),
  accepted_at      TIMESTAMPTZ,
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id)
);

CREATE INDEX idx_approved_tests_scope ON approved_tests(applicable_scope) WHERE deleted_at IS NULL;

-- ============================================================
-- SPECIES ARCHIVE  (shared by Engineers + Scientists)
-- Flexible taxonomic + discovery fields stored as JSONB
-- ============================================================
CREATE TABLE species_archive (
  id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name                   TEXT NOT NULL,
  classification         TEXT NOT NULL,
  scope                  TEXT NOT NULL CHECK (scope IN ('plant', 'all_species')),
  -- detail holds: taxonomy (domain→species), natural_habitat, discovery context, etc.
  detail                 JSONB NOT NULL DEFAULT '{}',
  -- JSONB shape:
  -- {
  --   "taxonomy": { "domain":"…","kingdom":"…","phylum":"…","class":"…",
  --                 "order":"…","family":"…","genus":"…","species":"…" },
  --   "natural_habitat": "…",
  --   "location_of_discovery": "…",
  --   "date_of_discovery": "YYYY-MM-DD",
  --   "description": "…"
  -- }
  related_experiment_ids UUID[],        -- array of experiment IDs
  created_by             UUID REFERENCES users(id),
  created_at             TIMESTAMPTZ DEFAULT NOW(),
  deleted_at             TIMESTAMPTZ,
  deleted_by             UUID REFERENCES users(id)
);

CREATE INDEX idx_species_archive_scope ON species_archive(scope) WHERE deleted_at IS NULL;

-- Junction: which species are targeted by a test
CREATE TABLE approved_test_species (
  test_id            UUID REFERENCES approved_tests(id),
  species_archive_id UUID REFERENCES species_archive(id),
  PRIMARY KEY (test_id, species_archive_id)
);

-- ============================================================
-- EXPERIMENTS  (shared by Engineers + Scientists — scoped by type)
-- ============================================================
CREATE TABLE experiments (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title       TEXT NOT NULL,
  type        TEXT NOT NULL CHECK (type IN (
    'agricultural', 'biological_engineering',
    'chemical', 'physical', 'biology_observation'
  )),
  -- Flexible proposal fields (introduction, problem statement, research questions, hypotheses)
  metadata    JSONB NOT NULL DEFAULT '{}',
  -- JSONB shape:
  -- {
  --   "introduction": "…",
  --   "problem_statement": "…",
  --   "research_questions": "…",
  --   "hypotheses": "…",
  --   "methodology": "…",
  --   "expected_outcomes": "…",
  --   "related_objects": ["uuid", …],
  --   "location": "…"
  -- }
  proposed_by UUID REFERENCES users(id),
  status      TEXT NOT NULL DEFAULT 'proposed' CHECK (status IN (
    'proposed','approved','active','conclusion_requested','closed','rejected'
  )),
  approved_by UUID REFERENCES users(id),
  approved_at TIMESTAMPTZ,
  closed_at   TIMESTAMPTZ,
  created_at  TIMESTAMPTZ DEFAULT NOW(),
  deleted_at  TIMESTAMPTZ,
  deleted_by  UUID REFERENCES users(id)
);

CREATE INDEX idx_experiments_type     ON experiments(type)        WHERE deleted_at IS NULL;
CREATE INDEX idx_experiments_status   ON experiments(status)      WHERE deleted_at IS NULL;
CREATE INDEX idx_experiments_proposer ON experiments(proposed_by) WHERE deleted_at IS NULL;

-- ============================================================
-- EXPERIMENT DAILY LOGS  (shared by Engineers + Scientists)
-- ============================================================
CREATE TABLE experiment_daily_logs (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  experiment_id    UUID NOT NULL REFERENCES experiments(id),
  log_date         DATE NOT NULL,
  rag_status       TEXT CHECK (rag_status IN ('red','amber','green')),
  completed_actions TEXT,
  pending_actions  TEXT,
  -- collected_data: flexible tabular data (e.g. cell density tables)
  collected_data   JSONB,
  created_by       UUID REFERENCES users(id),
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id)
);

CREATE INDEX idx_exp_logs_experiment ON experiment_daily_logs(experiment_id) WHERE deleted_at IS NULL;

-- Log attendees (multi-select, session-scoped)
CREATE TABLE experiment_log_attendees (
  log_id  UUID REFERENCES experiment_daily_logs(id),
  user_id UUID REFERENCES users(id),
  PRIMARY KEY (log_id, user_id)
);

-- Tests performed in a log session
CREATE TABLE experiment_log_tests (
  log_id  UUID REFERENCES experiment_daily_logs(id),
  test_id UUID REFERENCES approved_tests(id),
  PRIMARY KEY (log_id, test_id)
);

-- ============================================================
-- TEST PROPOSALS  (pending approval; approved → approved_tests)
-- ============================================================
CREATE TABLE test_proposals (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  proposed_by   UUID REFERENCES users(id),
  -- proposal_data holds: name, goal, procedure, species_scope, category, apparatuses, required_data, justification
  proposal_data JSONB NOT NULL,
  -- JSONB shape:
  -- {
  --   "name": "…", "goal": "…", "procedure": "…",
  --   "species_scope": "…", "category": ["physical","psychological","other"],
  --   "apparatuses": "…", "required_data": "…", "justification": "…"
  -- }
  status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
  reviewer_note TEXT,
  created_at    TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_test_proposals_status ON test_proposals(status);

-- ============================================================
-- FINAL OBJECT DOCUMENTS  (per-discipline, approval-gated)
-- Maps to UC-PH-08, UC-CH-07, UC-BIO-04
-- ============================================================
CREATE TABLE final_object_documents (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  experiment_id UUID REFERENCES experiments(id),
  type          TEXT NOT NULL CHECK (type IN ('matter','physical_object','species')),
  -- Flexible document fields per type
  document_data JSONB NOT NULL DEFAULT '{}',
  submitted_by  UUID REFERENCES users(id),
  status        TEXT NOT NULL DEFAULT 'pending_approval'
    CHECK (status IN ('pending_approval','approved','rejected')),
  reviewer_note TEXT,
  approved_by   UUID REFERENCES users(id),
  approved_at   TIMESTAMPTZ,
  created_at    TIMESTAMPTZ DEFAULT NOW(),
  deleted_at    TIMESTAMPTZ,
  deleted_by    UUID REFERENCES users(id)
);

CREATE INDEX idx_final_docs_experiment ON final_object_documents(experiment_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_final_docs_status     ON final_object_documents(status)        WHERE deleted_at IS NULL;

-- ============================================================
-- SCIENCE ARCHIVE  (matter, physical objects, species)
-- Covers: UC-PH-03, UC-CH-01, UC-BIO-01
-- ============================================================
CREATE TABLE science_archive (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  type           TEXT NOT NULL CHECK (type IN ('matter','physical_object','species')),
  name           TEXT NOT NULL,
  classification TEXT,
  -- detail holds all flexible fields per type
  detail         JSONB NOT NULL DEFAULT '{}',
  -- For type = 'matter':
  -- {
  --   "item_category": "…", "chemical_formula": "…",
  --   "date_of_origin": "…", "location_of_origin": "…",
  --   "status": "…", "description": "…",
  --   "proposal_experiment_id": "uuid", "acceptance_date": "…"
  -- }
  -- For type = 'physical_object':
  -- {
  --   "key_physical_characteristics": "…", "classification": "…",
  --   "date_of_origin": "…", "location_of_origin": "…",
  --   "description": "…", "proposal_experiment_id": "uuid"
  -- }
  -- For type = 'species':
  -- {
  --   "taxonomy": { "domain":"…", … },
  --   "natural_habitat": "…", "location_of_discovery": "…",
  --   "date_of_discovery": "…", "behavioral_characteristics": "…",
  --   "physical_characteristics": "…", "description": "…"
  -- }
  storage_path   TEXT,             -- optional: Supabase Storage path for images/documents
  experiment_id  UUID REFERENCES experiments(id),
  submitted_by   UUID REFERENCES users(id),
  approved_by    UUID REFERENCES users(id),
  created_at     TIMESTAMPTZ DEFAULT NOW(),
  deleted_at     TIMESTAMPTZ,
  deleted_by     UUID REFERENCES users(id)
);

CREATE INDEX idx_science_archive_type ON science_archive(type) WHERE deleted_at IS NULL;

-- ============================================================
-- MATH RESULTS  (UC-MA-01)
-- ============================================================
CREATE TABLE math_results (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id          UUID REFERENCES tasks(id),
  submitted_by     UUID REFERENCES users(id),
  -- content stored as JSONB: LaTeX source, rendered notes, workings
  content          JSONB NOT NULL,
  -- JSONB shape:
  -- {
  --   "content_latex": "\\frac{…}{…}",
  --   "workings": "Step 1: …",
  --   "calculations_area": "…"
  -- }
  -- Optional: exported PDF stored in Supabase Storage
  pdf_storage_path TEXT,           -- bucket: rusa-math, path: math-results/{result_id}/result.pdf
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id)
);

CREATE INDEX idx_math_results_task ON math_results(task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_math_results_user ON math_results(submitted_by) WHERE deleted_at IS NULL;

