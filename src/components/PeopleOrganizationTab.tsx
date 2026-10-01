import React, { useEffect, useMemo, useState } from 'react';
import {
  Activity,
  Building2,
  CalendarCheck2,
  ChevronDown,
  ChevronRight,
  ClipboardCheck,
  Clock3,
  GitBranch,
  MapPin,
  MoreHorizontal,
  Plus,
  Search,
  ShieldCheck,
  Target,
  Users,
  UserRound,
  X,
  Zap,
} from 'lucide-react';

type EmploymentType = 'Chính thức' | 'Thử việc' | 'Học việc' | 'Part-time';
type Attendance = 'Đang làm' | 'Đã ra về' | 'Đi muộn' | 'Nghỉ phép' | 'Vắng' | 'Chưa chấm công';

type Department = {
  id: string;
  name: string;
  shortName: string;
  owner: string;
  ownerTitle: string;
  charter: string;
  responsibilities: string[];
  kpis: string[];
  headcount: number;
  openRoles: number;
  criticality: 'Core' | 'Growth' | 'Control';
  status: 'Active' | 'Scaling' | 'Forming';
};

type Person = {
  id: string;
  name: string;
  title: string;
  departmentId: string;
  level: string;
  employmentType: EmploymentType;
  attendance: Attendance;
  location: string;
  workMode: 'Văn phòng' | 'Hybrid' | 'Remote' | 'Chưa cấu hình';
  managerId: string | null;
  shift: string;
  checkedInAt?: string;
  checkedOutAt?: string;
  avatar: string;
};

const seedDepartments: Department[] = [
  {
    id: 'exec',
    name: 'Executive Office',
    shortName: 'EXEC',
    owner: 'Alex Morgan',
    ownerTitle: 'Chief Executive Officer',
    charter: 'Định hướng chiến lược, phân bổ nguồn lực và chịu trách nhiệm P&L toàn công ty.',
    responsibilities: ['Strategy & OKR', 'Capital allocation', 'Executive governance', 'Cross-functional decisions'],
    kpis: ['Company EBITDA', 'Runway', 'OKR attainment'],
    headcount: 3,
    openRoles: 0,
    criticality: 'Control',
    status: 'Active',
  },
  {
    id: 'ops',
    name: 'Operations',
    shortName: 'OPS',
    owner: 'Sofia Nguyen',
    ownerTitle: 'Chief Operating Officer',
    charter: 'Biến chiến lược thành vận hành lặp lại, SLA rõ ràng và năng suất đo được.',
    responsibilities: ['Process excellence', 'Delivery operations', 'Workforce capacity', 'Vendor operations'],
    kpis: ['SLA attainment', 'Throughput', 'Cost / unit'],
    headcount: 4,
    openRoles: 1,
    criticality: 'Core',
    status: 'Scaling',
  },
  {
    id: 'finance',
    name: 'Finance & Treasury',
    shortName: 'FIN',
    owner: 'Daniel Tran',
    ownerTitle: 'Chief Financial Officer',
    charter: 'Bảo toàn tiền mặt, accounting integrity, payroll readiness và financial control.',
    responsibilities: ['Accounting', 'Treasury', 'Payroll control', 'Management reporting'],
    kpis: ['Cash accuracy', 'Close cycle', 'Runway'],
    headcount: 3,
    openRoles: 1,
    criticality: 'Control',
    status: 'Active',
  },
  {
    id: 'tech',
    name: 'Technology & Product',
    shortName: 'TECH',
    owner: 'Maya Patel',
    ownerTitle: 'Chief Technology Officer',
    charter: 'Xây dựng platform, data, security và product capabilities phục vụ tăng trưởng dài hạn.',
    responsibilities: ['Platform engineering', 'Security & reliability', 'Product delivery', 'Data systems'],
    kpis: ['Uptime', 'Release quality', 'Lead time'],
    headcount: 3,
    openRoles: 2,
    criticality: 'Core',
    status: 'Scaling',
  },
  {
    id: 'people',
    name: 'People & Culture',
    shortName: 'PEOPLE',
    owner: 'Linh Pham',
    ownerTitle: 'People & Culture Director',
    charter: 'Quản trị talent lifecycle, organization design, policy, attendance và employee experience.',
    responsibilities: ['Recruitment', 'Employment lifecycle', 'Attendance & policy', 'Performance & development'],
    kpis: ['Time-to-fill', 'Retention', 'Attendance integrity'],
    headcount: 3,
    openRoles: 1,
    criticality: 'Control',
    status: 'Active',
  },
  {
    id: 'growth',
    name: 'Growth & Commercial',
    shortName: 'GROWTH',
    owner: 'Jordan Lee',
    ownerTitle: 'Chief Growth Officer',
    charter: 'Tạo pipeline, tăng trưởng demand và chuyển đổi thành doanh thu được kiểm chứng.',
    responsibilities: ['Demand generation', 'Sales pipeline', 'Partnerships', 'Revenue experiments'],
    kpis: ['Qualified pipeline', 'Conversion', 'Gross revenue'],
    headcount: 0,
    openRoles: 3,
    criticality: 'Growth',
    status: 'Forming',
  },
];

const seedPeople: Person[] = [
  { id: 'p01', name: 'Alex Morgan', title: 'Chief Executive Officer', departmentId: 'exec', level: 'L9 · Executive', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Hybrid', managerId: null, shift: '08:30–17:30', checkedInAt: '08:21', avatar: 'AM' },
  { id: 'p02', name: 'Sofia Nguyen', title: 'Chief Operating Officer', departmentId: 'ops', level: 'L8 · Director', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Văn phòng', managerId: 'p01', shift: '08:30–17:30', checkedInAt: '08:26', avatar: 'SN' },
  { id: 'p03', name: 'Daniel Tran', title: 'Chief Financial Officer', departmentId: 'finance', level: 'L8 · Director', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Hybrid', managerId: 'p01', shift: '08:30–17:30', checkedInAt: '08:34', avatar: 'DT' },
  { id: 'p04', name: 'Maya Patel', title: 'Chief Technology Officer', departmentId: 'tech', level: 'L8 · Director', employmentType: 'Chính thức', attendance: 'Đã ra về', location: 'Singapore', workMode: 'Hybrid', managerId: 'p01', shift: '09:00–18:00', checkedInAt: '08:57', checkedOutAt: '16:48', avatar: 'MP' },
  { id: 'p05', name: 'Linh Pham', title: 'People & Culture Director', departmentId: 'people', level: 'L7 · Director', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Văn phòng', managerId: 'p02', shift: '08:30–17:30', checkedInAt: '08:17', avatar: 'LP' },
  { id: 'p06', name: 'Omar Wilson', title: 'Engineering Manager', departmentId: 'tech', level: 'L7 · Manager', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Bangkok', workMode: 'Remote', managerId: 'p04', shift: '09:00–18:00', checkedInAt: '08:52', avatar: 'OW' },
  { id: 'p07', name: 'Hailey Kim', title: 'Talent Partner', departmentId: 'people', level: 'L5 · Specialist', employmentType: 'Thử việc', attendance: 'Đi muộn', location: 'Ho Chi Minh City', workMode: 'Văn phòng', managerId: 'p05', shift: '08:30–17:30', checkedInAt: '09:03', avatar: 'HK' },
  { id: 'p08', name: 'Ethan Le', title: 'Senior Backend Engineer', departmentId: 'tech', level: 'L6 · Senior', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Da Nang', workMode: 'Hybrid', managerId: 'p06', shift: '09:00–18:00', checkedInAt: '08:49', avatar: 'EL' },
  { id: 'p09', name: 'Nora Bui', title: 'People Operations Coordinator', departmentId: 'people', level: 'L4 · Coordinator', employmentType: 'Học việc', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Văn phòng', managerId: 'p05', shift: '08:30–17:30', checkedInAt: '08:31', avatar: 'NB' },
  { id: 'p10', name: 'Marcus Ho', title: 'Financial Controller', departmentId: 'finance', level: 'L6 · Senior', employmentType: 'Chính thức', attendance: 'Nghỉ phép', location: 'Ho Chi Minh City', workMode: 'Hybrid', managerId: 'p03', shift: '08:30–17:30', avatar: 'MH' },
  { id: 'p11', name: 'Grace Chen', title: 'Product Operations Lead', departmentId: 'ops', level: 'L6 · Lead', employmentType: 'Chính thức', attendance: 'Đang làm', location: 'Singapore', workMode: 'Hybrid', managerId: 'p02', shift: '09:00–18:00', checkedInAt: '08:46', avatar: 'GC' },
  { id: 'p12', name: 'Noah Vo', title: 'Operations Apprentice', departmentId: 'ops', level: 'L2 · Apprentice', employmentType: 'Học việc', attendance: 'Đang làm', location: 'Ho Chi Minh City', workMode: 'Văn phòng', managerId: 'p11', shift: '08:30–17:30', checkedInAt: '08:29', avatar: 'NV' },
  { id: 'p13', name: 'Isabella Wong', title: 'Growth Specialist', departmentId: 'growth', level: 'L4 · Specialist', employmentType: 'Chính thức', attendance: 'Vắng', location: 'Kuala Lumpur', workMode: 'Remote', managerId: null, shift: '09:00–18:00', avatar: 'IW' },
];

const attendanceTone: Record<Attendance, string> = {
  'Đang làm': 'bg-emerald-500/10 text-emerald-300 border-emerald-500/20',
  'Đã ra về': 'bg-slate-500/10 text-slate-300 border-slate-500/20',
  'Đi muộn': 'bg-amber-500/10 text-amber-300 border-amber-500/20',
  'Nghỉ phép': 'bg-violet-500/10 text-violet-300 border-violet-500/20',
  Vắng: 'bg-rose-500/10 text-rose-300 border-rose-500/20',
  'Chưa chấm công': 'bg-slate-500/10 text-slate-400 border-slate-500/20',
};

const employmentTone: Record<EmploymentType, string> = {
  'Chính thức': 'bg-emerald-500/10 text-emerald-300 border-emerald-500/20',
  'Thử việc': 'bg-amber-500/10 text-amber-300 border-amber-500/20',
  'Học việc': 'bg-blue-500/10 text-blue-300 border-blue-500/20',
  'Part-time': 'bg-violet-500/10 text-violet-300 border-violet-500/20',
};

const deptTone: Record<Department['criticality'], string> = {
  Core: 'border-blue-500/20 bg-blue-500/10 text-blue-300',
  Growth: 'border-cyan-500/20 bg-cyan-500/10 text-cyan-300',
  Control: 'border-violet-500/20 bg-violet-500/10 text-violet-300',
};

const cx = (...v: Array<string | false | null | undefined>) => v.filter(Boolean).join(' ');

type OrganizationWorkspaceResponse = {
  company_id: string;
  currency: string;
  departments: Array<{
    id: string;
    company_id: string;
    parent_department_id?: string | null;
    code: string;
    name: string;
    charter: string;
    responsibilities: string[];
    kpis: string[];
    owner_employee_id?: string | null;
    monthly_budget_minor: number | string;
    currency: string;
    lifecycle: string;
    criticality: string;
    formation_reason?: string | null;
  }>;
  employees: Array<{
    employee_id: string;
    company_id: string;
    name: string;
    title: string;
    department_id?: string | null;
    team_id?: string | null;
    manager_id?: string | null;
    employment_type: string;
    employment_level: string;
    joined_at_epoch?: number | null;
    status: string;
  }>;
  attendance: Array<{
    employee_id: string;
    status: string;
    shift_start: string;
    shift_end: string;
    check_in_at_epoch?: number | null;
    check_out_at_epoch?: number | null;
  }>;
  source: string;
};

const mapEmploymentType = (value: string): EmploymentType => {
  switch (value) {
    case 'PROBATION': return 'Thử việc';
    case 'APPRENTICE': return 'Học việc';
    case 'PART_TIME': return 'Part-time';
    default: return 'Chính thức';
  }
};

const mapAttendance = (value?: string): Attendance => {
  switch (value) {
    case 'PRESENT': return 'Đang làm';
    case 'REMOTE': return 'Đang làm';
    case 'CHECKED_OUT': return 'Đã ra về';
    case 'LATE': return 'Đi muộn';
    case 'LEAVE': return 'Nghỉ phép';
    case 'ABSENT': return 'Vắng';
    default: return 'Chưa chấm công';
  }
};

const mapCriticality = (value: string): Department['criticality'] => {
  switch (value) {
    case 'CONTROL': return 'Control';
    case 'GROWTH': return 'Growth';
    default: return 'Core';
  }
};

const mapDepartmentStatus = (value: string): Department['status'] => {
  switch (value) {
    case 'PROPOSED': return 'Forming';
    case 'SCALING': return 'Scaling';
    default: return value === 'CLOSED' || value === 'PAUSED' ? 'Forming' : 'Active';
  }
};

const formatEpochTime = (epoch?: number | null) =>
  epoch == null
    ? undefined
    : new Intl.DateTimeFormat('vi-VN', {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
        timeZone: 'Asia/Ho_Chi_Minh',
      }).format(new Date(epoch * 1000));


export const PeopleOrganizationTab: React.FC<{ aiAgentCount?: number }> = ({ aiAgentCount = 0 }) => {
  const [livePeople, setLivePeople] = useState<Person[]>([]);
  const [dataSource, setDataSource] = useState<'DATABASE' | 'DATABASE_EMPTY' | 'DEMO_FALLBACK'>('DEMO_FALLBACK');
  const [liveCompanyId, setLiveCompanyId] = useState('');
  const [liveCurrency, setLiveCurrency] = useState('USD');
  const [showEmployeeForm, setShowEmployeeForm] = useState(false);
  const [savingEmployee, setSavingEmployee] = useState(false);
  const [newEmployee, setNewEmployee] = useState({ name: '', title: '', departmentId: '', managerId: '', employmentType: 'OFFICIAL', employmentLevel: 'L4', monthlyCost: '0' });
  const [view, setView] = useState<'overview' | 'departments' | 'employees' | 'attendance'>('overview');
  const [selectedDepartmentId, setSelectedDepartmentId] = useState('people');
  const [selectedEmployeeId, setSelectedEmployeeId] = useState('');
  const [search, setSearch] = useState('');
  const [employment, setEmployment] = useState<'All' | EmploymentType>('All');
  const [showAdvisor, setShowAdvisor] = useState(true);
  const [departmentList, setDepartmentList] = useState<Department[]>(seedDepartments);
  const [departmentActivationPending, setDepartmentActivationPending] = useState(false);

  const people = dataSource === 'DEMO_FALLBACK' ? seedPeople : livePeople;
  const selectedDepartment = departmentList.find((department) => department.id === selectedDepartmentId) ?? departmentList[0];
  const selectedEmployee = selectedEmployeeId ? people.find((person) => person.id === selectedEmployeeId) : undefined;

  useEffect(() => {
    let cancelled = false;
    const loadOrganization = async () => {
      try {
        const workDate = new Date().toISOString().slice(0, 10);
        const response = await fetch(`/api/organization?work_date=${workDate}`);
        if (!response.ok) throw new Error('organization API unavailable');
        const data = (await response.json()) as OrganizationWorkspaceResponse;
        if (cancelled) return;
        setLiveCompanyId(data.company_id);
        setLiveCurrency(data.currency);

        const attendanceByEmployee = new Map(data.attendance.map((record) => [record.employee_id, record]));
        const nextPeople: Person[] = data.employees.map((employee) => {
          const attendance = attendanceByEmployee.get(employee.employee_id);
          return {
            id: employee.employee_id,
            name: employee.name,
            title: employee.title,
            departmentId: employee.department_id ?? 'unassigned',
            level: employee.employment_level,
            employmentType: mapEmploymentType(employee.employment_type),
            attendance: mapAttendance(attendance?.status),
            location: 'Chưa cấu hình',
            workMode: attendance?.status === 'REMOTE' ? 'Remote' : 'Chưa cấu hình',
            managerId: employee.manager_id ?? null,
            shift: attendance ? `${attendance.shift_start}–${attendance.shift_end}` : 'Theo attendance policy',
            checkedInAt: formatEpochTime(attendance?.check_in_at_epoch),
            checkedOutAt: formatEpochTime(attendance?.check_out_at_epoch),
            avatar: employee.name.split(' ').map((part) => part[0]).join('').slice(0, 2).toUpperCase(),
          };
        });

        const nextDepartments: Department[] = data.departments.map((department) => ({
          id: department.id,
          name: department.name,
          shortName: department.code,
          owner: department.owner_employee_id ? 'Assigned owner' : 'Unassigned',
          ownerTitle: department.owner_employee_id ? 'Accountable owner' : 'Owner required',
          charter: department.charter,
          responsibilities: department.responsibilities,
          kpis: department.kpis,
          headcount: nextPeople.filter((person) => person.departmentId === department.id).length,
          openRoles: 0,
          criticality: mapCriticality(department.criticality),
          status: mapDepartmentStatus(department.lifecycle),
        }));

        setLivePeople(nextPeople);
        if (nextDepartments.length) {
          setDepartmentList(nextDepartments);
          setSelectedDepartmentId((current) => nextDepartments.some((item) => item.id === current) ? current : nextDepartments[0].id);
        }
        setDataSource('DATABASE');
      } catch {
        if (!cancelled) setDataSource('DEMO_FALLBACK');
      }
    };

    void loadOrganization();
    return () => {
      cancelled = true;
    };
  }, []);

  const rootEmployee = people.find((person) => person.managerId === null) ?? people[0];
  const rootDirectReports = rootEmployee ? people.filter((person) => person.managerId === rootEmployee.id) : [];

  const presentCount = people.filter((person) => person.attendance === 'Đang làm' || person.attendance === 'Đã ra về').length;
  const lateCount = people.filter((person) => person.attendance === 'Đi muộn').length;
  const leaveCount = people.filter((person) => person.attendance === 'Nghỉ phép').length;

  const filteredPeople = useMemo(
    () => people.filter((person) => {
      const query = search.trim().toLowerCase();
      const matchesQuery = !query || person.name.toLowerCase().includes(query) || person.title.toLowerCase().includes(query);
      const matchesEmployment = employment === 'All' || person.employmentType === employment;
      return matchesQuery && matchesEmployment;
    }),
    [search, employment],
  );

  return (
    <div className="space-y-5 pb-14">
      <section className="rounded-2xl border border-slate-800 bg-[radial-gradient(circle_at_top_right,_rgba(59,130,246,0.13),_transparent_34%),linear-gradient(135deg,rgba(15,23,42,0.98),rgba(15,23,42,0.95))] p-5 lg:p-6">
        <div className="flex flex-col xl:flex-row xl:items-end xl:justify-between gap-5">
          <div>
            <div className="flex items-center gap-2 text-[10px] uppercase tracking-[0.2em] text-blue-300 font-semibold"><ShieldCheck className="w-3.5 h-3.5" />People Operating System</div>
            <div className="flex flex-wrap items-center gap-3 mt-2">
              <h1 className="text-2xl lg:text-[30px] font-semibold tracking-tight text-white">People, Departments & Organization</h1>
              <span className="rounded-full border border-emerald-500/20 bg-emerald-500/10 px-2 py-1 text-[10px] font-semibold text-emerald-300">ENTERPRISE HR</span>
              <span className={cx('rounded-full border px-2 py-1 text-[10px] font-semibold', dataSource === 'DATABASE' ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-300' : 'border-amber-500/20 bg-amber-500/10 text-amber-300')}>
                {dataSource === 'DATABASE' ? 'LIVE DATABASE' : dataSource === 'DATABASE_EMPTY' ? 'DATABASE · CHƯA CÓ RECORDS' : 'DEMO FALLBACK'}
              </span>
            </div>
            <p className="mt-2 max-w-4xl text-sm leading-6 text-slate-400">Công ty được vận hành theo mô hình <span className="text-slate-200 font-medium">Company → Department → Team → Employee</span>. Mỗi department có charter, owner, KPI và ranh giới trách nhiệm; mọi thay đổi tổ chức phải có governance record.</p>
          </div>
          <div className="flex items-center gap-2">
            <button className="inline-flex items-center gap-2 rounded-xl border border-slate-700 bg-slate-900/80 px-3.5 py-2.5 text-xs font-semibold text-slate-200"><CalendarCheck2 className="w-4 h-4" /> Hôm nay</button>
            <button onClick={() => { setNewEmployee((current) => ({ ...current, departmentId: selectedDepartment?.id ?? departmentList[0]?.id ?? "" })); setShowEmployeeForm(true); }} className="inline-flex items-center gap-2 rounded-xl bg-blue-600 px-3.5 py-2.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-500"><Plus className="w-4 h-4" /> Thêm nhân sự</button>
          </div>
        </div>

        <div className="mt-6 grid grid-cols-2 lg:grid-cols-5 gap-2.5">
          {([
            { label: 'Departments', value: departmentList.length, note: 'Active + forming', Icon: Building2 },
            { label: 'Human workforce', value: people.length, note: 'Employee records', Icon: Users },
            { label: 'Present', value: presentCount, note: 'Today', Icon: CalendarCheck2 },
            { label: 'Exceptions', value: lateCount, note: 'Late arrivals', Icon: Clock3 },
            { label: 'AI workforce', value: aiAgentCount, note: 'Separate AI layer', Icon: Zap },
          ] as const).map(({ label, value, note, Icon }) => (
            <div key={label} className="rounded-xl border border-slate-800 bg-slate-950/55 px-3.5 py-3">
              <div className="flex items-center justify-between">
                <span className="text-[10px] uppercase tracking-[0.12em] text-slate-500">{label}</span>
                <Icon className="w-4 h-4 text-slate-500" />
              </div>
              <div className="mt-2 text-2xl font-semibold font-mono text-white">{value}</div>
              <div className="mt-1 text-[10px] text-slate-500">{note}</div>
            </div>
          ))}
        </div>
      </section>

      <div className="flex items-center gap-1 rounded-xl border border-slate-800 bg-slate-950/60 p-1 w-fit">
        {[
          ['overview', 'Tổng quan'],
          ['departments', 'Phòng ban'],
          ['employees', 'Nhân viên'],
          ['attendance', 'Chấm công'],
        ].map(([id, label]) => (
          <button key={id} onClick={() => setView(id as typeof view)} className={cx('px-3.5 py-2 rounded-lg text-xs font-semibold transition-colors', view === id ? 'bg-slate-800 text-white' : 'text-slate-500 hover:text-slate-200')}>
            {label}
          </button>
        ))}
      </div>

      {view !== 'attendance' && (
        <section className="rounded-2xl border border-slate-800 bg-slate-900/55 overflow-hidden">
          <div className="px-4 py-3.5 border-b border-slate-800 flex items-center justify-between">
            <div className="flex items-center gap-2"><GitBranch className="w-4 h-4 text-blue-300" /><div><h2 className="text-sm font-semibold text-white">Organization Tree</h2><p className="text-[10px] text-slate-500">Company → Department → Team → Employee</p></div></div>
            <span className="text-[10px] text-slate-500">{departmentList.filter((d) => d.status !== 'Forming').length} operational departments</span>
          </div>
          <div className="p-4 overflow-x-auto">
            <div className="min-w-[1080px]">
              <div className="flex justify-center">
                <button onClick={() => setSelectedDepartmentId('exec')} className={cx('w-[300px] rounded-2xl border px-4 py-4 text-left', selectedDepartmentId === 'exec' ? 'border-blue-500/40 bg-blue-500/[0.06]' : 'border-slate-700 bg-slate-950/80')}>
                  <div className="flex items-center gap-3"><div className="w-11 h-11 rounded-xl bg-gradient-to-br from-blue-500/25 to-violet-500/20 border border-blue-400/20 flex items-center justify-center"><Building2 className="w-5 h-5 text-blue-200" /></div><div><div className="text-[10px] uppercase tracking-[0.15em] text-blue-300">Corporate</div><div className="text-sm font-semibold text-white">Executive Office</div><div className="text-[10px] text-slate-500">CEO · Governance · Strategy</div></div></div>
                  <div className="mt-3 flex items-center justify-between text-[10px]"><span className="text-slate-500">Direct departments</span><span className="text-slate-200 font-semibold">5</span></div>
                </button>
              </div>
              <div className="mx-auto h-7 w-px bg-slate-700"></div>
              <div className="relative">
                <div className="absolute left-[10%] right-[10%] top-0 h-px bg-slate-700"></div>
                <div className="grid grid-cols-5 gap-3">
                  {departmentList.filter((d) => d.id !== 'exec').map((department) => (
                    <button key={department.id} onClick={() => setSelectedDepartmentId(department.id)} className={cx('relative rounded-xl border p-3 text-left transition-all', selectedDepartmentId === department.id ? 'border-blue-500/40 bg-blue-500/[0.05]' : 'border-slate-800 bg-slate-950/65 hover:border-slate-700')}>
                      <div className="absolute -top-2 left-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-slate-950 border border-slate-700"></div>
                      <div className="flex items-center gap-2.5"><div className={cx('w-8 h-8 rounded-lg border border-slate-700 bg-slate-800 flex items-center justify-center text-[9px] font-bold', department.status === 'Forming' ? 'text-cyan-300' : 'text-slate-200')}>{department.shortName}</div><div className="min-w-0"><div className="text-[11px] font-semibold text-white truncate">{department.name}</div><div className="text-[9px] text-slate-500 truncate">{department.ownerTitle}</div></div></div>
                      <div className="mt-2 flex items-center justify-between"><span className={cx('text-[9px] px-1.5 py-0.5 rounded border', deptTone[department.criticality])}>{department.criticality}</span><span className="text-[9px] text-slate-500">{department.headcount} HC</span></div>
                    </button>
                  ))}
                </div>
              </div>

              <div className="grid grid-cols-5 gap-3 mt-4">
                {departmentList.filter((d) => d.id !== 'exec').map((department) => {
                  const departmentPeople = people.filter((person) => person.departmentId === department.id).slice(0, 3);
                  return (
                    <div key={department.id} className="rounded-xl border border-slate-800 bg-slate-950/45 p-2.5">
                      <div className="flex items-center justify-between text-[9px] text-slate-600 uppercase tracking-[0.1em]"><span>People</span><span>{department.openRoles ? '+' + department.openRoles + ' roles' : 'Covered'}</span></div>
                      <div className="mt-2 space-y-1.5">
                        {departmentPeople.map((person) => (
                          <button key={person.id} onClick={() => setSelectedEmployeeId(person.id)} className="w-full flex items-center gap-2 rounded-lg border border-slate-800 bg-slate-900/45 px-2.5 py-2 text-left hover:border-slate-700">
                            <div className="w-7 h-7 rounded-lg bg-slate-800 border border-slate-700 flex items-center justify-center text-[9px] font-bold text-slate-300">{person.avatar}</div>
                            <div className="min-w-0 flex-1"><div className="text-[10px] font-medium text-slate-200 truncate">{person.name}</div><div className="text-[9px] text-slate-600 truncate">{person.title}</div></div>
                            <ChevronRight className="w-3 h-3 text-slate-600" />
                          </button>
                        ))}
                        {department.status === 'Forming' && <div className="rounded-lg border border-dashed border-cyan-500/20 bg-cyan-500/[0.03] px-2.5 py-2 text-[9px] text-cyan-300">Department forming · 3 critical roles open</div>}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </section>
      )}

      {(view === 'overview' || view === 'departments') && (
        <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,1.35fr)_minmax(360px,0.65fr)] gap-4">
          <section className="rounded-2xl border border-slate-800 bg-slate-900/55 overflow-hidden">
            <div className="px-4 py-3.5 border-b border-slate-800 flex items-center justify-between"><div><h2 className="text-sm font-semibold text-white">Department Charters</h2><p className="text-[10px] text-slate-500 mt-0.5">Mỗi phòng ban có trách nhiệm, owner và KPI riêng</p></div><ClipboardCheck className="w-4 h-4 text-slate-500" /></div>
            <div className="p-4 grid grid-cols-1 md:grid-cols-2 gap-3">
              {departmentList.map((department) => (
                <button key={department.id} onClick={() => setSelectedDepartmentId(department.id)} className={cx('rounded-xl border p-3.5 text-left transition-all', selectedDepartmentId === department.id ? 'border-blue-500/35 bg-blue-500/[0.045]' : 'border-slate-800 bg-slate-950/50 hover:border-slate-700')}>
                  <div className="flex items-start justify-between gap-3">
                    <div><div className="flex items-center gap-2"><h3 className="text-xs font-semibold text-white">{department.name}</h3><span className={cx('text-[8px] px-1.5 py-0.5 rounded border', deptTone[department.criticality])}>{department.criticality}</span></div><div className="mt-1 text-[10px] text-slate-500">Owner · {department.owner} · {department.ownerTitle}</div></div>
                    <span className={cx('text-[9px] px-2 py-1 rounded-md border', department.status === 'Forming' ? 'border-cyan-500/20 bg-cyan-500/10 text-cyan-300' : department.status === 'Scaling' ? 'border-amber-500/20 bg-amber-500/10 text-amber-300' : 'border-emerald-500/20 bg-emerald-500/10 text-emerald-300')}>{department.status}</span>
                  </div>
                  <p className="mt-3 text-[10px] leading-5 text-slate-400">{department.charter}</p>
                  <div className="mt-3 flex flex-wrap gap-1.5">{department.responsibilities.map((item) => <span key={item} className="text-[9px] rounded-md border border-slate-800 bg-slate-950 px-2 py-1 text-slate-500">{item}</span>)}</div>
                  <div className="mt-3 pt-3 border-t border-slate-800/80 flex items-center justify-between text-[9px]"><span className="text-slate-500">{department.headcount} employees · {department.openRoles} open roles</span><span className="text-blue-300">View charter →</span></div>
                </button>
              ))}
            </div>
          </section>

          <section className="rounded-2xl border border-slate-800 bg-slate-900/55 overflow-hidden">
            <div className="px-4 py-3.5 border-b border-slate-800 flex items-center gap-2"><Target className="w-4 h-4 text-violet-300" /><div><h2 className="text-sm font-semibold text-white">Department Control</h2><p className="text-[10px] text-slate-500">Owner · accountability · KPI</p></div></div>
            <div className="p-4">
              <div className="rounded-xl border border-slate-800 bg-slate-950/55 p-3.5"><div className="text-[10px] uppercase tracking-[0.12em] text-slate-600">Selected department</div><div className="mt-1 text-lg font-semibold text-white">{selectedDepartment.name}</div><div className="text-[10px] text-slate-500 mt-1">{selectedDepartment.owner} · {selectedDepartment.ownerTitle}</div><div className="mt-3 rounded-lg border border-slate-800 bg-slate-900/40 p-3 text-[10px] leading-5 text-slate-400">{selectedDepartment.charter}</div></div>
              <div className="mt-3 space-y-2">{selectedDepartment.responsibilities.map((responsibility, index) => <div key={responsibility} className="flex items-center gap-2.5 rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2.5"><span className="w-5 h-5 rounded-full border border-slate-700 bg-slate-900 text-[9px] font-bold text-slate-500 flex items-center justify-center">{index + 1}</span><span className="text-[10px] text-slate-300">{responsibility}</span></div>)}</div>
              <div className="mt-3 grid grid-cols-3 gap-2">{selectedDepartment.kpis.map((kpi) => <div key={kpi} className="rounded-lg border border-slate-800 bg-slate-950/40 p-2.5"><div className="text-[9px] text-slate-600">KPI</div><div className="mt-1 text-[10px] font-semibold text-slate-300">{kpi}</div></div>)}</div>
              <button className="mt-3 w-full rounded-xl border border-slate-700 bg-slate-900 py-2.5 text-[10px] font-semibold text-slate-200 hover:bg-slate-800">Mở Department Charter</button>
            </div>
          </section>
        </div>
      )}

      {view === 'employees' && (
        <section className="rounded-2xl border border-slate-800 bg-slate-900/55 overflow-hidden">
          <div className="px-4 py-3.5 border-b border-slate-800 flex flex-wrap gap-3 items-center justify-between">
            <div><h2 className="text-sm font-semibold text-white">Employee Directory</h2><p className="text-[10px] text-slate-500 mt-0.5">{filteredPeople.length} employee records</p></div>
            <div className="flex items-center gap-2"><div className="relative"><Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-600" /><input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Tìm nhân viên…" className="w-52 rounded-xl border border-slate-800 bg-slate-950/70 pl-9 pr-3 py-2 text-[10px] text-white placeholder:text-slate-600 outline-none focus:border-blue-500/50" /></div><select value={employment} onChange={(event) => setEmployment(event.target.value as typeof employment)} className="rounded-xl border border-slate-800 bg-slate-950/70 px-2.5 py-2 text-[10px] text-slate-300 outline-none"><option value="All">Tất cả employment</option><option value="Chính thức">Chính thức</option><option value="Thử việc">Thử việc</option><option value="Học việc">Học việc</option><option value="Part-time">Part-time</option></select></div>
          </div>
          <div className="overflow-x-auto"><table className="w-full min-w-[980px] text-left"><thead><tr className="border-b border-slate-800 text-[9px] uppercase tracking-[0.12em] text-slate-600"><th className="px-4 py-3">Employee</th><th className="px-4 py-3">Department</th><th className="px-4 py-3">Level</th><th className="px-4 py-3">Employment</th><th className="px-4 py-3">Attendance</th><th className="px-4 py-3">Location</th><th className="px-4 py-3"></th></tr></thead><tbody>{filteredPeople.map((person) => <tr key={person.id} onClick={() => setSelectedEmployeeId(person.id)} className={cx('border-b border-slate-800/80 cursor-pointer', selectedEmployeeId === person.id ? 'bg-blue-500/[0.04]' : 'hover:bg-slate-950/55')}><td className="px-4 py-3"><div className="flex items-center gap-3"><div className="w-9 h-9 rounded-xl border border-slate-700 bg-slate-800 flex items-center justify-center text-[10px] font-bold text-slate-200">{person.avatar}</div><div><div className="text-xs font-semibold text-white">{person.name}</div><div className="text-[10px] text-slate-500">{person.title}</div></div></div></td><td className="px-4 py-3 text-[10px] text-slate-300">{departmentList.find((department) => department.id === person.departmentId)?.name}</td><td className="px-4 py-3 text-[10px] text-slate-500">{person.level}</td><td className="px-4 py-3"><span className={cx('text-[9px] px-2 py-1 rounded border', employmentTone[person.employmentType])}>{person.employmentType}</span></td><td className="px-4 py-3"><span className={cx('text-[9px] px-2 py-1 rounded border', attendanceTone[person.attendance])}>{person.attendance}</span></td><td className="px-4 py-3"><div className="flex items-center gap-1.5 text-[10px] text-slate-400"><MapPin className="w-3 h-3 text-slate-600" />{person.workMode}</div><div className="text-[9px] text-slate-600">{person.location}</div></td><td className="px-4 py-3 text-right"><MoreHorizontal className="w-4 h-4 text-slate-600" /></td></tr>)}</tbody></table></div>
        </section>
      )}

      {view === 'attendance' && (
        <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,1.4fr)_minmax(320px,0.6fr)] gap-4">
          <section className="rounded-2xl border border-slate-800 bg-slate-900/55 overflow-hidden">
            <div className="px-4 py-3.5 border-b border-slate-800 flex items-center justify-between"><div><h2 className="text-sm font-semibold text-white">Attendance Control Board</h2><p className="text-[10px] text-slate-500 mt-0.5">Daily timekeeping · shift · exceptions · manager ownership</p></div><span className="rounded-md border border-emerald-500/20 bg-emerald-500/10 px-2 py-1 text-[9px] font-semibold text-emerald-300">LIVE</span></div>
            <div className="p-4 grid grid-cols-1 md:grid-cols-2 gap-2.5">{people.map((person) => <button key={person.id} onClick={() => setSelectedEmployeeId(person.id)} className="flex items-center gap-3 rounded-xl border border-slate-800 bg-slate-950/50 p-3 text-left hover:border-slate-700"><div className="w-9 h-9 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-[10px] font-bold text-slate-300">{person.avatar}</div><div className="min-w-0 flex-1"><div className="flex items-center gap-2"><span className="text-[11px] font-semibold text-slate-200 truncate">{person.name}</span><span className={cx('text-[8px] px-1.5 py-0.5 rounded border', attendanceTone[person.attendance])}>{person.attendance}</span></div><div className="text-[9px] text-slate-600 mt-1">{person.shift} · {person.workMode}</div></div><div className="text-right shrink-0 text-[9px] font-mono text-slate-500">{person.checkedInAt ? 'IN ' + person.checkedInAt : '—'}<br />{person.checkedOutAt ? 'OUT ' + person.checkedOutAt : ''}</div></button>)}</div>
          </section>
          <section className="rounded-2xl border border-slate-800 bg-slate-900/55 p-4"><div className="flex items-center gap-2"><Activity className="w-4 h-4 text-emerald-300" /><div><h2 className="text-sm font-semibold text-white">Attendance Exceptions</h2><p className="text-[10px] text-slate-500">Cần manager review</p></div></div><div className="mt-4 space-y-2">{people.filter((person) => person.attendance === 'Đi muộn' || person.attendance === 'Vắng').map((person) => <div key={person.id} className="rounded-xl border border-amber-500/15 bg-amber-500/[0.035] p-3"><div className="flex items-center justify-between"><span className="text-[10px] font-semibold text-white">{person.name}</span><span className={cx('text-[8px] px-1.5 py-0.5 rounded border', attendanceTone[person.attendance])}>{person.attendance}</span></div><div className="mt-1 text-[9px] text-slate-500">{departmentList.find((department) => department.id === person.departmentId)?.name} · {person.shift}</div><button className="mt-2 text-[9px] font-semibold text-blue-300">Review exception →</button></div>)}</div></section>
        </div>
      )}

      {showAdvisor && (
        <section className="rounded-2xl border border-cyan-500/15 bg-cyan-500/[0.03] p-4">
          <div className="flex flex-col xl:flex-row xl:items-center xl:justify-between gap-4">
            <div className="flex items-start gap-3"><div className="w-9 h-9 rounded-xl border border-cyan-500/20 bg-cyan-500/10 flex items-center justify-center"><Zap className="w-4 h-4 text-cyan-300" /></div><div><div className="text-[10px] uppercase tracking-[0.14em] text-cyan-300 font-semibold">Organization Design Advisor</div><h3 className="text-sm font-semibold text-white mt-1">Growth & Commercial đang ở trạng thái “Forming”</h3><p className="text-[10px] text-slate-400 mt-1 max-w-3xl">Nhu cầu pipeline tăng nhưng chưa có owner đầy đủ. Hệ thống có thể <span className="text-slate-200 font-medium">đề xuất thành lập department</span>, xác định charter, budget ceiling, owner và open roles trước khi activation.</p></div></div>
            <div className="flex items-center gap-2"><button className="rounded-xl border border-slate-700 bg-slate-900 px-3 py-2 text-[10px] font-semibold text-slate-200">Xem nhu cầu</button><button onClick={() => setDepartmentActivationPending(true)} className="rounded-xl bg-cyan-600 px-3 py-2 text-[10px] font-semibold text-white hover:bg-cyan-500">Dự thảo department</button><button onClick={() => setShowAdvisor(false)} className="p-2 text-slate-600 hover:text-white"><X className="w-4 h-4" /></button></div>
          </div>
          <div className="mt-4 grid grid-cols-1 md:grid-cols-3 gap-2.5"><div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Trigger</div><div className="text-[10px] text-slate-300 mt-1">Pipeline ownership gap</div></div><div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Required capability</div><div className="text-[10px] text-slate-300 mt-1">Sales + partnerships + demand generation</div></div><div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Governance</div><div className="text-[10px] text-slate-300 mt-1">Charter → owner → budget → roles → activation</div></div></div>
        </section>
      )}

      {showEmployeeForm && (
        <div className="fixed inset-0 z-[66] bg-black/55 backdrop-blur-[2px] flex items-center justify-center p-4" onClick={() => setShowEmployeeForm(false)}>
          <form className="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-950 shadow-2xl overflow-hidden" onClick={(event) => event.stopPropagation()} onSubmit={async (event) => {
            event.preventDefault();
            if (dataSource !== 'DATABASE' || !liveCompanyId || !newEmployee.name.trim() || !newEmployee.title.trim() || !newEmployee.departmentId) return;
            setSavingEmployee(true);
            const employeeId = crypto.randomUUID();
            try {
              const response = await fetch('/api/organization/employees', {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                  input: {
                    employee: {
                      employee_id: employeeId,
                      company_id: liveCompanyId,
                      department_id: newEmployee.departmentId,
                      team_id: null,
                      manager_id: newEmployee.managerId || null,
                      title: newEmployee.title.trim(),
                      employment_type: newEmployee.employmentType,
                      employment_level: newEmployee.employmentLevel.trim() || 'L4',
                      joined_at_epoch: Math.floor(Date.now() / 1000),
                      status: 'ACTIVE',
                    },
                    name: newEmployee.name.trim(),
                    monthly_cost_minor: Number(newEmployee.monthlyCost || 0),
                    currency: liveCurrency,
                  },
                }),
              });
              if (!response.ok) throw new Error('employee create failed');
              setLivePeople((current) => [...current, {
                id: employeeId,
                name: newEmployee.name.trim(),
                title: newEmployee.title.trim(),
                departmentId: newEmployee.departmentId,
                level: newEmployee.employmentLevel.trim() || 'L4',
                employmentType: mapEmploymentType(newEmployee.employmentType),
                attendance: 'Chưa chấm công',
                location: 'Chưa cấu hình',
                workMode: 'Chưa cấu hình',
                managerId: newEmployee.managerId || null,
                shift: 'Theo attendance policy',
                avatar: newEmployee.name.trim().split(' ').map((part) => part[0]).join('').slice(0, 2).toUpperCase(),
              }]);
              setDepartmentList((current) => current.map((item) => item.id === newEmployee.departmentId ? { ...item, headcount: item.headcount + 1 } : item));
              setSelectedEmployeeId(employeeId);
              setShowEmployeeForm(false);
              setNewEmployee({ name: '', title: '', departmentId: newEmployee.departmentId, managerId: '', employmentType: 'OFFICIAL', employmentLevel: 'L4', monthlyCost: '0' });
            } catch (error) {
              console.error(error);
            } finally {
              setSavingEmployee(false);
            }
          }}>
            <div className="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
              <div><div className="text-[10px] uppercase tracking-[0.14em] text-blue-300">People onboarding</div><div className="mt-1 text-sm font-semibold text-white">Thêm nhân sự vào phòng ban</div></div>
              <button type="button" onClick={() => setShowEmployeeForm(false)} className="p-2 rounded-lg text-slate-500 hover:text-white hover:bg-slate-800"><X className="w-4 h-4" /></button>
            </div>
            <div className="p-5 grid grid-cols-2 gap-3">
              <label className="col-span-2"><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Họ tên</span><input required value={newEmployee.name} onChange={(event) => setNewEmployee((current) => ({ ...current, name: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[11px] text-white outline-none" /></label>
              <label className="col-span-2"><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Chức danh</span><input required value={newEmployee.title} onChange={(event) => setNewEmployee((current) => ({ ...current, title: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[11px] text-white outline-none" /></label>
              <label><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Phòng ban</span><select required value={newEmployee.departmentId} onChange={(event) => setNewEmployee((current) => ({ ...current, departmentId: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[10px] text-slate-300">{departmentList.map((department) => <option key={department.id} value={department.id}>{department.name}</option>)}</select></label>
              <label><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Manager</span><select value={newEmployee.managerId} onChange={(event) => setNewEmployee((current) => ({ ...current, managerId: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[10px] text-slate-300"><option value="">Không có</option>{people.filter((person) => person.id && liveCompanyId && person.id.length === 36).map((person) => <option key={person.id} value={person.id}>{person.name}</option>)}</select></label>
              <label><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Employment</span><select value={newEmployee.employmentType} onChange={(event) => setNewEmployee((current) => ({ ...current, employmentType: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[10px] text-slate-300"><option value="OFFICIAL">Chính thức</option><option value="PROBATION">Thử việc</option><option value="APPRENTICE">Học việc</option><option value="PART_TIME">Part-time</option><option value="CONTRACTOR">Contractor</option></select></label>
              <label><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Level</span><input value={newEmployee.employmentLevel} onChange={(event) => setNewEmployee((current) => ({ ...current, employmentLevel: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[11px] text-white outline-none" /></label>
              <label className="col-span-2"><span className="text-[9px] uppercase tracking-[0.1em] text-slate-600">Monthly cost (minor unit)</span><input type="number" min="0" value={newEmployee.monthlyCost} onChange={(event) => setNewEmployee((current) => ({ ...current, monthlyCost: event.target.value }))} className="mt-1 w-full rounded-xl border border-slate-800 bg-slate-900 px-3 py-2.5 text-[11px] text-white outline-none" /></label>
            </div>
            <div className="px-5 py-4 border-t border-slate-800 flex justify-end gap-2">
              <button type="button" onClick={() => setShowEmployeeForm(false)} className="rounded-xl border border-slate-700 bg-slate-900 px-4 py-2.5 text-[10px] font-semibold text-slate-300">Hủy</button>
              <button type="submit" disabled={savingEmployee || dataSource !== 'DATABASE' || !liveCompanyId} className="rounded-xl bg-blue-600 px-4 py-2.5 text-[10px] font-semibold text-white disabled:opacity-40">{savingEmployee ? 'Đang tạo…' : 'Tạo nhân sự'}</button>
            </div>
          </form>
        </div>
      )}

      {departmentActivationPending && (
        <div className="fixed inset-0 z-[65] bg-black/55 backdrop-blur-[2px] flex items-center justify-center p-4" onClick={() => setDepartmentActivationPending(false)}>
          <div className="w-full max-w-lg rounded-2xl border border-cyan-500/15 bg-slate-950 shadow-2xl overflow-hidden" onClick={(event) => event.stopPropagation()}>
            <div className="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
              <div><div className="text-[10px] uppercase tracking-[0.14em] text-cyan-300">Department proposal</div><div className="mt-1 text-sm font-semibold text-white">Growth & Commercial</div></div>
              <button onClick={() => setDepartmentActivationPending(false)} className="p-2 rounded-lg text-slate-500 hover:text-white hover:bg-slate-800"><X className="w-4 h-4" /></button>
            </div>
            <div className="p-5 space-y-4">
              <div className="rounded-xl border border-slate-800 bg-slate-900/55 p-3.5 text-[10px] leading-5 text-slate-400">Activation chỉ được ghi nhận sau khi charter, owner, budget ceiling và critical roles đã được xác định. Đây là organization change có audit trail, không phải tự ý bypass governance.</div>
              <div className="grid grid-cols-2 gap-2.5">
                <div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Charter</div><div className="mt-1 text-[10px] text-slate-300">Revenue pipeline & partnerships</div></div>
                <div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Initial team</div><div className="mt-1 text-[10px] text-slate-300">Head of Sales + 2 specialists</div></div>
                <div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Owner</div><div className="mt-1 text-[10px] text-slate-300">Jordan Lee · CGO</div></div>
                <div className="rounded-xl border border-slate-800 bg-slate-950/60 p-3"><div className="text-[9px] text-slate-600">Budget ceiling</div><div className="mt-1 text-[10px] text-slate-300">$12,000 / month</div></div>
              </div>
              <div className="flex gap-2">
                <button onClick={() => setDepartmentActivationPending(false)} className="flex-1 rounded-xl border border-slate-700 bg-slate-900 py-2.5 text-[10px] font-semibold text-slate-300">Để review</button>
                <button
                  onClick={() => {
                    setDepartmentList((current) => current.some((item) => item.id === 'growth') ? current.map((item) => item.id === 'growth' ? { ...item, status: 'Active' as const } : item) : [...current, { id: 'growth', name: 'Growth & Commercial', shortName: 'GROWTH', owner: 'Jordan Lee', ownerTitle: 'Chief Growth Officer', charter: 'Tạo pipeline, tăng trưởng demand và chuyển đổi thành doanh thu được kiểm chứng.', responsibilities: ['Demand generation', 'Sales pipeline', 'Partnerships', 'Revenue experiments'], kpis: ['Qualified pipeline', 'Conversion', 'Gross revenue'], headcount: 0, openRoles: 3, criticality: 'Growth' as const, status: 'Active' as const }]);
                    setSelectedDepartmentId('growth');
                    setDepartmentActivationPending(false);
                    setView('departments');
                  }}
                  className="flex-1 rounded-xl bg-cyan-600 py-2.5 text-[10px] font-semibold text-white hover:bg-cyan-500"
                >
                  Kích hoạt với governance
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {selectedEmployee && (view === 'employees' || view === 'attendance' || view === 'overview') && (
        <div className="fixed inset-0 z-[70] bg-black/50 backdrop-blur-[2px] flex justify-end" onClick={() => setSelectedEmployeeId('')}>
          <aside className="h-full w-full max-w-md bg-slate-950 border-l border-slate-800 shadow-2xl overflow-y-auto" onClick={(event) => event.stopPropagation()}>
            <div className="sticky top-0 z-10 flex items-center justify-between border-b border-slate-800 bg-slate-950/95 px-5 py-4"><div><div className="text-[10px] uppercase tracking-[0.14em] text-blue-300/80">Employee profile</div><div className="text-sm font-semibold text-white mt-1">Hồ sơ nhân sự</div></div><button onClick={() => setSelectedEmployeeId('')} className="rounded-lg p-2 text-slate-500 hover:bg-slate-800 hover:text-white"><X className="w-4 h-4" /></button></div>
            <div className="p-5 space-y-5">
              <div className="rounded-2xl border border-slate-800 bg-slate-900/75 p-4"><div className="flex items-center gap-3"><div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-blue-500/20 to-violet-500/15 border border-blue-400/15 flex items-center justify-center text-lg font-semibold text-blue-100">{selectedEmployee.avatar}</div><div><h3 className="text-lg font-semibold text-white">{selectedEmployee.name}</h3><p className="text-xs text-slate-400 mt-0.5">{selectedEmployee.title}</p><div className="mt-2 flex gap-1.5"><span className={cx('text-[9px] px-2 py-1 rounded border', employmentTone[selectedEmployee.employmentType])}>{selectedEmployee.employmentType}</span><span className="text-[9px] px-2 py-1 rounded border border-slate-700 bg-slate-950 text-slate-500">{selectedEmployee.level}</span></div></div></div></div>
              <div className="grid grid-cols-2 gap-2.5">{[['Department', departments.find((department) => department.id === selectedEmployee.departmentId)?.name],['Manager', people.find((person) => person.id === selectedEmployee.managerId)?.name ?? 'CEO'],['Work mode', selectedEmployee.workMode],['Location', selectedEmployee.location],['Shift', selectedEmployee.shift],['Attendance', selectedEmployee.attendance]].map(([label, value]) => <div key={label} className="rounded-xl border border-slate-800 bg-slate-900/45 px-3 py-2.5"><div className="text-[9px] uppercase tracking-[0.1em] text-slate-600">{label}</div><div className="text-[10px] text-slate-300 mt-1 truncate">{value}</div></div>)}</div>
              <div className="rounded-2xl border border-blue-500/15 bg-blue-500/[0.03] p-4"><div className="flex items-center gap-2"><UserRound className="w-4 h-4 text-blue-300" /><div><div className="text-xs font-semibold text-white">Employment lifecycle</div><div className="text-[10px] text-slate-500 mt-0.5">Một hồ sơ · employment status · attendance</div></div></div><div className="mt-4 space-y-2">{['Profile verified', selectedEmployee.employmentType, 'Reporting line assigned', 'Attendance policy assigned', selectedEmployee.employmentType === 'Chính thức' ? 'Review completed' : 'Probation / apprenticeship in progress'].map((item, index) => <div key={item} className="flex items-center gap-3"><div className="w-6 h-6 rounded-full border border-slate-700 bg-slate-900 flex items-center justify-center text-[9px] font-bold text-slate-500">{index + 1}</div><span className="text-[10px] text-slate-300">{item}</span></div>)}</div></div>
              <div className="grid grid-cols-2 gap-2">
                <button type="button" disabled={!liveCompanyId || dataSource !== 'DATABASE'} onClick={async () => {
                  const [shiftStart, shiftEnd] = selectedEmployee.shift.includes('–') ? selectedEmployee.shift.split('–') : ['08:30', '17:30'];
                  const checkIn = Math.floor(Date.now() / 1000);
                  const response = await fetch('/api/organization/attendance', {
                    method: 'POST',
                    headers: { 'content-type': 'application/json' },
                    body: JSON.stringify({ attendance: { id: crypto.randomUUID(), company_id: liveCompanyId, employee_id: selectedEmployee.id, work_date: localCompanyDate(), status: 'PRESENT', shift_start: shiftStart, shift_end: shiftEnd, check_in_at_epoch: checkIn, check_out_at_epoch: null, source: 'company-os-ui', exception_reason: null } }),
                  });
                  if (response.ok) setLivePeople((current) => current.map((person) => person.id === selectedEmployee.id ? { ...person, attendance: 'Đang làm', checkedInAt: formatEpochTime(checkIn), checkedOutAt: undefined } : person));
                }} className="rounded-xl border border-emerald-500/20 bg-emerald-500/10 px-3 py-2.5 text-[10px] font-semibold text-emerald-300 disabled:opacity-40">Check-in</button>
                <button type="button" disabled={!liveCompanyId || dataSource !== 'DATABASE'} onClick={async () => {
                  const [shiftStart, shiftEnd] = selectedEmployee.shift.includes('–') ? selectedEmployee.shift.split('–') : ['08:30', '17:30'];
                  const checkOut = Math.floor(Date.now() / 1000);
                  const response = await fetch('/api/organization/attendance', {
                    method: 'POST',
                    headers: { 'content-type': 'application/json' },
                    body: JSON.stringify({ attendance: { id: crypto.randomUUID(), company_id: liveCompanyId, employee_id: selectedEmployee.id, work_date: localCompanyDate(), status: 'CHECKED_OUT', shift_start: shiftStart, shift_end: shiftEnd, check_in_at_epoch: null, check_out_at_epoch: checkOut, source: 'company-os-ui', exception_reason: null } }),
                  });
                  if (response.ok) setLivePeople((current) => current.map((person) => person.id === selectedEmployee.id ? { ...person, attendance: 'Đã ra về', checkedOutAt: formatEpochTime(checkOut) } : person));
                }} className="rounded-xl border border-slate-700 bg-slate-900 px-3 py-2.5 text-[10px] font-semibold text-slate-200 disabled:opacity-40">Check-out</button>
              </div>
            </div>
          </aside>
        </div>
      )}

      <div className="flex items-center justify-between px-1 text-[9px] text-slate-600"><div className="flex items-center gap-2"><ShieldCheck className="w-3.5 h-3.5" />Payroll, employment changes and department activation should remain approval-gated and auditable.</div><div>Human workforce {people.length} · AI workforce {aiAgentCount}</div></div>
    </div>
  );
};
