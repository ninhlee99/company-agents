CREATE TABLE IF NOT EXISTS departments (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  parent_department_id uuid REFERENCES departments(id),
  code text NOT NULL,
  name text NOT NULL,
  charter text NOT NULL,
  responsibilities jsonb NOT NULL DEFAULT '[]'::jsonb,
  kpis jsonb NOT NULL DEFAULT '[]'::jsonb,
  owner_employee_id uuid REFERENCES employees(id),
  monthly_budget_minor numeric(39,0) NOT NULL DEFAULT 0 CHECK (monthly_budget_minor >= 0),
  currency char(3) NOT NULL,
  lifecycle text NOT NULL CHECK (lifecycle IN ('PROPOSED','ACTIVE','SCALING','PAUSED','CLOSED')),
  criticality text NOT NULL CHECK (criticality IN ('CORE','GROWTH','CONTROL')),
  formation_reason text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, code),
  UNIQUE(company_id, name)
);

ALTER TABLE employees
  ADD COLUMN IF NOT EXISTS department_id uuid REFERENCES departments(id),
  ADD COLUMN IF NOT EXISTS team_id uuid,
  ADD COLUMN IF NOT EXISTS manager_id uuid REFERENCES employees(id),
  ADD COLUMN IF NOT EXISTS employment_type text NOT NULL DEFAULT 'OFFICIAL'
    CHECK (employment_type IN ('OFFICIAL','PROBATION','APPRENTICE','PART_TIME','CONTRACTOR')),
  ADD COLUMN IF NOT EXISTS employment_level text NOT NULL DEFAULT 'L4',
  ADD COLUMN IF NOT EXISTS joined_at_epoch bigint;

CREATE INDEX IF NOT EXISTS idx_employees_org
  ON employees(company_id, department_id, manager_id, status);

CREATE TABLE IF NOT EXISTS teams (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  department_id uuid NOT NULL REFERENCES departments(id),
  parent_team_id uuid REFERENCES teams(id),
  name text NOT NULL,
  charter text NOT NULL,
  owner_employee_id uuid REFERENCES employees(id),
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, department_id, name)
);

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'employees_team_fk'
  ) THEN
    ALTER TABLE employees
      ADD CONSTRAINT employees_team_fk
      FOREIGN KEY (team_id) REFERENCES teams(id);
  END IF;
END $$;

CREATE TABLE IF NOT EXISTS employee_attendance (
  id uuid PRIMARY KEY,
  company_id uuid NOT NULL REFERENCES companies(id),
  employee_id uuid NOT NULL REFERENCES employees(id),
  work_date date NOT NULL,
  status text NOT NULL CHECK (status IN ('PRESENT','REMOTE','LATE','LEAVE','ABSENT','CHECKED_OUT')),
  shift_start time NOT NULL,
  shift_end time NOT NULL,
  check_in_at timestamptz,
  check_out_at timestamptz,
  source text NOT NULL,
  exception_reason text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(company_id, employee_id, work_date)
);

CREATE INDEX IF NOT EXISTS idx_employee_attendance_daily
  ON employee_attendance(company_id, work_date, status);

CREATE INDEX IF NOT EXISTS idx_departments_parent
  ON departments(company_id, parent_department_id);
