# People Operating Model

## Organization hierarchy

The company is represented as:

**Company → Department → Team → Employee**

Every employee should belong to a department, may belong to a team, and should have a manager unless they are the root executive for the company.

## Department standard

Each department has:
- a stable code and name;
- a charter describing why the department exists;
- explicit responsibilities;
- measurable KPIs;
- an accountable owner;
- a monthly budget ceiling;
- a lifecycle: PROPOSED, ACTIVE, SCALING, PAUSED or CLOSED;
- a criticality class: CORE, GROWTH or CONTROL.

A department should not be treated as an unstructured employee bucket. Its charter is the boundary of accountable work.

## Employee standard

Employee records carry:
- employment type: OFFICIAL, PROBATION, APPRENTICE, PART_TIME or CONTRACTOR;
- employment level;
- department and optional team assignment;
- manager / reporting line;
- employment status;
- joined-at timestamp.

Payroll, contract changes and termination remain approval-gated and auditable.

## Attendance standard

Attendance is stored as a company-scoped daily record with:
- work date;
- status;
- shift window;
- check-in / check-out;
- source;
- exception reason.

Supported states are PRESENT, REMOTE, LATE, LEAVE, ABSENT and CHECKED_OUT.

The system should treat attendance as an operational record, not as an inference from login activity alone.

## Organization design automation

The organization design advisor may evaluate a formation signal using:
- required capabilities;
- capacity gap percentage;
- number of sustained cycles;
- proposed monthly budget ceiling.

A department is considered for formation only when the gap is at least 25% and has remained for at least 3 cycles.

When the department already exists and is active, the result is **ScaleExisting** rather than creating a duplicate department.

When no active department exists, the result is **FormDepartment**.

Activation should follow:

**Need signal → Charter → Owner → Budget ceiling → Critical roles → Governance record → Activation**

Automation may draft and prepare the structure, but the activation event must remain auditable and bounded by the organization's approval policy.


## HR operating baseline

The Company OS bootstraps an HR catalog at startup:
- standard job positions with level, allowed employment types and responsibilities;
- standard attendance policies in Asia/Ho_Chi_Minh with explicit shifts, workdays and grace periods;
- attendance policy assignments for employees;
- append-only employment lifecycle events with actor and approval references.

Employee onboarding is position-driven. The selected position supplies the authoritative title/level and must belong to the selected department. A successful onboarding transaction also assigns the default attendance policy and records the initial employment lifecycle.

Attendance writes resolve the employee's active policy for the work date. Shift windows are taken from the policy, not trusted from the browser. For present/remote check-ins, lateness is derived server-side from local check-in time plus policy grace.

No legal employee identity, compensation amount, or physical attendance is fabricated by bootstrap. Those records require real operator input or an external attendance integration.
