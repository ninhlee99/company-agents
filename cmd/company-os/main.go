package main

import (
	"html/template"
	"log"
	"net/http"
	"time"
)

type DashboardData struct {
	Title string
	Time  string
	Status string
	Agents []AgentView
}

type AgentView struct {
	Name   string
	Status string
}

var page = template.Must(template.New("dashboard").Parse(`
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{{.Title}}</title>
  <style>
    body { font-family: system-ui,sans-serif; max-width: 1100px; margin: 40px auto; padding: 0 20px; }
    .grid { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:16px; }
    .card { border:1px solid #ddd; border-radius:12px; padding:18px; }
    table { width:100%; border-collapse:collapse; } th,td { text-align:left; padding:10px; border-bottom:1px solid #eee; }
    .ok { color:green; } .muted { color:#666; }
  </style>
</head>
<body>
  <h1>Company OS</h1>
  <p class="muted">Go runtime • {{.Time}}</p>
  <div class="grid">
    <div class="card"><strong>System</strong><div>{{.Status}}</div></div>
    <div class="card"><strong>Agents</strong><div>{{len .Agents}}</div></div>
    <div class="card"><strong>Mode</strong><div>Control plane</div></div>
  </div>
  <h2>Agents</h2>
  <div class="card">
    <table>
      <tr><th>Agent</th><th>Status</th></tr>
      {{range .Agents}}<tr><td>{{.Name}}</td><td class="ok">{{.Status}}</td></tr>{{end}}
    </table>
  </div>
</body>
</html>
`))

func main() {
	agents := []AgentView{
		{"Governor", "READY"},
		{"CEO", "READY"},
		{"CFO", "READY"},
		{"COO", "READY"},
		{"Analyst", "READY"},
		{"Experiment", "READY"},
		{"Growth", "READY"},
		{"Content", "READY"},
		{"Recruiter", "READY"},
	}

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/" {
			http.NotFound(w, r)
			return
		}
		data := DashboardData{
			Title: "Company OS",
			Time: time.Now().UTC().Format(time.RFC3339),
			Status: "ONLINE (bootstrap)",
			Agents: agents,
		}
		if err := page.Execute(w, data); err != nil {
			http.Error(w, "template error", http.StatusInternalServerError)
		}
	})

	http.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok
"))
	})

	log.Println("Company OS listening on http://localhost:8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
