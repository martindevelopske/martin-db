use std::sync::{Arc, RwLock};

use axum::{
    Json, Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use martin_db::{
    Database,
    engine::ExecutionResult,
    parser::parse,
    storage::{load_from_disk, save_to_disk},
};
use serde::{Deserialize, Serialize};

struct AppStateInner {
    db: Database,
}

type SharedState = Arc<RwLock<AppStateInner>>;

#[derive(Deserialize)]
struct QueryRequest {
    sql: String,
}

#[derive(Deserialize, Serialize)]
struct QueryResponse {
    message: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    // 1. Load DB
    let db = load_from_disk().unwrap_or_else(|_| Database::new());
    let state = Arc::new(RwLock::new(AppStateInner { db }));

    // 2. Define Routes
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/query", post(query_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Database Web Demo running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// Handler to execute SQL queries sent from the UI
async fn query_handler(
    State(state): State<SharedState>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let mut state_guard = state.write().unwrap();

    match parse(&payload.sql) {
        Ok(stmt) => {
            let is_mutation = matches!(
                stmt,
                martin_db::parser::Statement::CreateTable { .. }
                    | martin_db::parser::Statement::Insert { .. }
            );

            match state_guard.db.execute(stmt) {
                Ok(result) => {
                    if is_mutation {
                        let _ = save_to_disk(&state_guard.db);
                    }
                    match result {
                        ExecutionResult::Message(m) => Json(QueryResponse {
                            message: m,
                            headers: vec![],
                            rows: vec![],
                            error: None,
                        }),
                        ExecutionResult::Data { headers, rows } => Json(QueryResponse {
                            message: "Success".into(),
                            headers,
                            rows: rows
                                .into_iter()
                                .map(|r| r.into_iter().map(|v| format!("{:?}", v)).collect())
                                .collect(),
                            error: None,
                        }),
                    }
                }
                Err(e) => Json(QueryResponse {
                    message: "Execution Error".into(),
                    headers: vec![],
                    rows: vec![],
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(QueryResponse {
            message: "Syntax Error".into(),
            headers: vec![],
            rows: vec![],
            error: Some(e),
        }),
    }
}

// A simple HTML UI with JavaScript to interact with our DB
async fn ui_handler() -> Html<&'static str> {
    Html(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Pesapal Database Demo</title>
        <style>
            body { font-family: sans-serif; margin: 40px; background: #f4f4f9; }
            .container { max-width: 800px; margin: auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            input { width: 80%; padding: 10px; }
            button { padding: 10px 20px; cursor: pointer; background: #28a745; color: white; border: none; border-radius: 4px; }
            table { width: 100%; border-collapse: collapse; margin-top: 20px; }
            th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
            th { background: #f8f9fa; }
            .error { color: red; margin-top: 10px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>Web Interface</h1>
            <p>Run SQL queries against my Rust DB:</p>
            <input type="text" id="sqlInput" placeholder="SELECT * FROM users..." value="SELECT * FROM devs JOIN teams ON team_id = id">
            <button onclick="runQuery()">Execute</button>
            <div id="error" class="error"></div>
            <div id="result"></div>
        </div>

        <script>
            async function runQuery() {
                const sql = document.getElementById('sqlInput').value;
                const res = await fetch('/query', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ sql })
                });
                const data = await res.json();
                
                if (data.error) {
                    document.getElementById('error').innerText = data.error;
                    document.getElementById('result').innerHTML = '';
                } else {
                    document.getElementById('error').innerText = '';
                    let html = `<h3>${data.message}</h3>`;
                    if (data.headers.length > 0) {
                        html += '<table><thead><tr>' + data.headers.map(h => `<th>${h}</th>`).join('') + '</tr></thead><tbody>';
                        html += data.rows.map(row => '<tr>' + row.map(cell => `<td>${cell}</td>`).join('') + '</tr>').join('');
                        html += '</tbody></table>';
                    }
                    document.getElementById('result').innerHTML = html;
                }
            }
        </script>
    </body>
    </html>
    "#,
    )
}
