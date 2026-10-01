CREATE TABLE IF NOT EXISTS job_positions (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  department_id uuid NOT NULL,
  team_id uuid,
  code text NOT NULL,
  title text NOT NULL,
  level text NOT NULL,
  employment_types jsonb NOT NULL DEFAULT '[]'::jsonb,
  responsibilities jsonb NOT NULL DEFAULT '[]'::jsonb,
  monthly_cost_min_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (monthly_cost_min_minor >= 0),
  monthly_cost_max_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (monthly_cost_max_minor >= monthly_cost_min_minor),
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, code),
  UNIQUE(company_id, department_id, title)
);

CREATE TABLE IF NOT EXISTS attendance_policies (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  code text NOT NULL,
  name text NOT NULL,
  timezone text NOT NULL DEFAULT 'Asia/Ho_Chi_Minh',
  shift_start time NOT NULL,
  shift_end time NOT NULL,
  grace_minutes integer NOT NULL DEFAULT 0 CHECK (grace_minutes >= 0 AND grace_minutes <= 240),
  work_days smallint[] NOT NULL DEFAULT ARRAY[1,2,3,4,5],
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS employee_attendance_policy_assignments (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  employee_id uuid NOT NULL,
  policy_id uuid NOT NULL,
  effective_from date NOT NULL,
  effective_to date,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK (effective_to IS NULL OR effective_to >= effective_from)
);

CREATE TABLE IF NOT EXISTS employment_lifecycle_events (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  employee_id uuid NOT NULL,
  event_type text NOT NULL CHECK (event_type IN (
    'HIRED','PROBATION_STARTED','PROBATION_PASSED','APPRENTICESHIP_STARTED',
    'APPOINTED','PROMOTED','TRANSFERRED','MANAGER_CHANGED','SUSPENDED',
    'REINSTATED','TERMINATED'
  )),
  effective_at timestamptz NOT NULL,
  position_id uuid,
  department_id uuid,
  team_id uuid,
  manager_id uuid,
  notes text,
  approval_reference text,
  actor_id text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, employee_id, event_type, effective_at)
);

ALTER TABLE employees ADD COLUMN IF NOT EXISTS position_id uuid;

ALTER TABLE job_positions
  ADD CONSTRAINT job_positions_department_same_company_fk
  FOREIGN KEY (company_id, department_id) REFERENCES departments(company_id, id);

ALTER TABLE job_positions
  ADD CONSTRAINT job_positions_team_same_company_fk
  FOREIGN KEY (company_id, team_id) REFERENCES teams(company_id, id);

ALTER TABLE employees
  ADD CONSTRAINT employees_position_same_company_fk
  FOREIGN KEY (company_id, position_id) REFERENCES job_positions(company_id, id);

ALTER TABLE employee_attendance_policy_assignments
  ADD CONSTRAINT attendance_assignment_employee_same_company_fk
  FOREIGN KEY (company_id, employee_id) REFERENCES employees(company_id, id);

ALTER TABLE employee_attendance_policy_assignments
  ADD CONSTRAINT attendance_assignment_policy_same_company_fk
  FOREIGN KEY (company_id, policy_id) REFERENCES attendance_policies(company_id, id);

ALTER TABLE employment_lifecycle_events
  ADD CONSTRAINT employment_event_employee_same_company_fk
  FOREIGN KEY (company_id, employee_id) REFERENCES employees(company_id, id);

ALTER TABLE employment_lifecycle_events
  ADD CONSTRAINT employment_event_position_same_company_fk
  FOREIGN KEY (company_id, position_id) REFERENCES job_positions(company_id, id);

ALTER TABLE employment_lifecycle_events
  ADD CONSTRAINT employment_event_department_same_company_fk
  FOREIGN KEY (company_id, department_id) REFERENCES departments(company_id, id);

ALTER TABLE employment_lifecycle_events
  ADD CONSTRAINT employment_event_team_same_company_fk
  FOREIGN KEY (company_id, team_id) REFERENCES teams(company_id, id);

ALTER TABLE employment_lifecycle_events
  ADD CONSTRAINT employment_event_manager_same_company_fk
  FOREIGN KEY (company_id, manager_id) REFERENCES employees(company_id, id);

CREATE INDEX IF NOT EXISTS idx_job_positions_company_department ON job_positions(company_id, department_id, active);
CREATE INDEX IF NOT EXISTS idx_attendance_policies_company_active ON attendance_policies(company_id, active);
CREATE INDEX IF NOT EXISTS idx_attendance_assignment_employee ON employee_attendance_policy_assignments(company_id, employee_id, effective_from DESC);
CREATE INDEX IF NOT EXISTS idx_employment_events_employee ON employment_lifecycle_events(company_id, employee_id, effective_at DESC);

CREATE OR REPLACE FUNCTION reject_employment_lifecycle_mutation() RETURNS trigger
LANGUAGE plpgsql AS $hr$
BEGIN
  RAISE EXCEPTION 'employment_lifecycle_events are append-only';
END;
$hr$;

DROP TRIGGER IF EXISTS employment_lifecycle_events_no_update ON employment_lifecycle_events;
DROP TRIGGER IF EXISTS employment_lifecycle_events_no_delete ON employment_lifecycle_events;

CREATE TRIGGER employment_lifecycle_events_no_update
BEFORE UPDATE ON employment_lifecycle_events
FOR EACH ROW EXECUTE FUNCTION reject_employment_lifecycle_mutation();

CREATE TRIGGER employment_lifecycle_events_no_delete
BEFORE DELETE ON employment_lifecycle_events
FOR EACH ROW EXECUTE FUNCTION reject_employment_lifecycle_mutation();
