use anyhow::{anyhow, Result};
use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use chrono::{NaiveTime};
use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
/// Matéria vinda do frontend
#[derive(Debug, Clone, Deserialize)]
struct SubjectInput {
    name: String,
    priority: u32,
    min_minutes: u32,
}

/// Matéria usada internamente
#[derive(Debug, Clone)]
struct Subject {
    name: String,
    priority: u32,
    min_minutes: u32,
}

/// Bloco interno (heurística local)
#[derive(Debug)]
struct Block {
    subject_name: String,
    minutes: u32,
}

/// Requisição do frontend para gerar plano
#[derive(Debug, Deserialize)]
struct PlanRequest {
    total_hours: f32,
    start_time: String, // "08:00"
    subjects: Vec<SubjectInput>,
    focus: Option<String>,
    use_ai: bool,
}

/// Bloco final enviado para o frontend
#[derive(Debug, Serialize)]
struct UiBlock {
    start: String,
    end: String,
    subject: String,
    minutes: u32,
}

/// Resposta enviada ao frontend
#[derive(Debug, Serialize)]
struct PlanResponse {
    mode: String,         // "ai" ou "local"
    blocks: Vec<UiBlock>, // blocos calculados
}

/// Bloco retornado pela IA
#[derive(Deserialize, Debug)]
struct AiBlock {
    start: String,
    end: String,
    subject: String,
}

/// Plano retornado pela IA
#[derive(Deserialize, Debug)]
struct AiPlan {
    blocks: Vec<AiBlock>,
}

/// Estrutura da Responses API
#[derive(Deserialize, Debug)]
struct ResponseRoot {
    output: Vec<ResponseOutputItem>,
}

#[derive(Deserialize, Debug)]
struct ResponseOutputItem {
    #[serde(rename = "type")]
    item_type: String,
    content: Vec<ResponseContentItem>,
}

#[derive(Deserialize, Debug)]
struct ResponseContentItem {
    #[serde(rename = "type")]
    content_type: String, // "output_text"
    text: String,
}


#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("🧪 Inicializando servidor...");
    println!("PORT = {:?}", env::var("PORT"));

    let mut port: u16 = 10000;
    if let Ok(port_var) = env::var("PORT") {
        if let Ok(parsed) = port_var.parse() {
            port = parsed;
        }
    }

    let api_router = Router::new().route("/api/plan", post(create_plan_handler));
    let app = Router::new()
        .nest("/", api_router)
        .fallback_service(ServeDir::new("static"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Servindo em http://{addr}");
    println!("📁 CWD = {:?}", env::current_dir());

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        eprintln!("❌ Falha ao fazer bind em {addr}: {err}");
        err
    })?;
    println!("✅ Listener pronto em {addr}");

    axum::serve(listener, app).await.map_err(|err| {
        eprintln!("❌ Erro ao servir em {addr}: {err}");
        err
    })?;

    Ok(())
}

/// Handler HTTP: recebe JSON, tenta IA, cai em fallback local se der erro
async fn create_plan_handler(Json(req): Json<PlanRequest>) -> impl IntoResponse {
    match create_plan(req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(err) => {
            eprintln!("[ERRO] create_plan: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    }
}

/// Lógica principal: IA (se possível) + fallback local
async fn create_plan(req: PlanRequest) -> Result<PlanResponse> {
    // Converter para estrutura interna
    let subjects: Vec<Subject> = req
        .subjects
        .iter()
        .map(|s| Subject {
            name: s.name.clone(),
            priority: s.priority,
            min_minutes: s.min_minutes,
        })
        .collect();

    let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M")
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(8, 0, 0).unwrap());

    let total_minutes = (req.total_hours * 60.0) as u32;

    // Tentar IA se use_ai = true
    if req.use_ai {
        match call_openai_and_get_plan(&subjects, req.total_hours, start_time, req.focus.clone())
            .await
        {
            Ok(ai_plan) => {
                // Converter AiPlan -> UiBlock (calculando minutos pela diferença de horários)
                let mut ui_blocks = Vec::new();
                for b in ai_plan.blocks {
                    let start = NaiveTime::parse_from_str(&b.start, "%H:%M").unwrap_or(start_time);
                    let end = NaiveTime::parse_from_str(&b.end, "%H:%M").unwrap_or(start);
                    let minutes = end.signed_duration_since(start).num_minutes().max(0) as u32;

                    ui_blocks.push(UiBlock {
                        start: b.start,
                        end: b.end,
                        subject: b.subject,
                        minutes,
                    });
                }

                return Ok(PlanResponse {
                    mode: "ai".to_string(),
                    blocks: ui_blocks,
                });
            }
            Err(e) => {
                eprintln!("[AVISO] Falha ao chamar IA, caindo para modo local: {e}");
            }
        }
    }

    // Fallback local
    let local_blocks = generate_schedule(subjects, total_minutes, req.focus.clone());
    let ui_blocks = blocks_to_ui_blocks(local_blocks, start_time);

    Ok(PlanResponse {
        mode: "local".to_string(),
        blocks: ui_blocks,
    })
}

/// Monta o prompt e chama a API da OpenAI (Responses API)
async fn call_openai_and_get_plan(
    subjects: &Vec<Subject>,
    total_hours: f32,
    start_time: NaiveTime,
    focus: Option<String>,
) -> Result<AiPlan> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("Defina OPENAI_API_KEY no .env ou nas variáveis de ambiente"))?;

    let client = Client::new();

    let mut subjects_desc = String::new();
    for s in subjects {
        subjects_desc.push_str(&format!(
            "- {} (prioridade {}, mínimo {} min)\n",
            s.name, s.priority, s.min_minutes
        ));
    }

    let focus_name = focus.unwrap_or_else(|| "nenhum".to_string());

    let prompt = format!(
        "Você é um planejador de estudos especialista.\n\
        Gere uma rotina de estudo detalhada PARA HOJE em blocos de horário, em português do Brasil.\n\n\
        Dados:\n\
        - Horas totais de estudo: {:.2}\n\
        - Horário de início: {}\n\
        - Matérias (nome, prioridade 1-5, minutos mínimos):\n\
        {}\n\
        - Matéria foco principal: {}\n\n\
        Regras:\n\
        - Respeite o tempo total de estudo.\n\
        - Dê mais tempo para matérias de maior prioridade e para a matéria foco.\n\
        - Não crie blocos menores que 20 minutos.\n\
        - Evite mais de 90 minutos seguidos na mesma matéria.\n\
        - Pode incluir pequenas pausas (matéria \"Pausa\"), mas conte essas pausas dentro do total de horas.\n\
        - Use horários coerentes a partir do horário de início.\n\n\
        Formato de resposta:\n\
        - Responda APENAS com um JSON VÁLIDO, sem texto antes ou depois.\n\
        - NÃO use ``` nem markdown.\n\
        - Estrutura exata:\n\
        {{\n\
          \"blocks\": [\n\
            {{ \"start\": \"08:00\", \"end\": \"09:00\", \"subject\": \"Matemática\" }}\n\
          ]\n\
        }}",
        total_hours,
        start_time.format("%H:%M"),
        subjects_desc,
        focus_name
    );

    let body = json!({
        "model": "gpt-4.1-mini",
        "input": prompt,
        "max_output_tokens": 512
    });

    let resp = client
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(anyhow!("Erro da OpenAI: {}", text));
    }

    let api_resp: ResponseRoot = resp.json().await?;

    let first_output = api_resp
        .output
        .get(0)
        .ok_or_else(|| anyhow!("Resposta da API não contém 'output'"))?;

    let first_content = first_output
        .content
        .iter()
        .find(|c| c.content_type == "output_text")
        .ok_or_else(|| anyhow!("Resposta da API não contém 'output_text'"))?;

    let text = &first_content.text;

    let plan: AiPlan = serde_json::from_str(text).map_err(|e| {
        anyhow!(
            "Falha ao parsear JSON da IA: {}\nConteúdo recebido:\n{}",
            e,
            text
        )
    })?;

    Ok(plan)
}

/// Heurística local (mesma ideia da versão CLI)
fn generate_schedule(
    mut subjects: Vec<Subject>,
    total_minutes: u32,
    focus: Option<String>,
) -> Vec<Block> {
    if subjects.is_empty() || total_minutes == 0 {
        return Vec::new();
    }

    // Bônus de foco
    if let Some(ref focus_name) = focus {
        for s in &mut subjects {
            if s.name.to_lowercase() == focus_name.to_lowercase() {
                s.priority *= 2;
            }
        }
    }

    // Soma dos mínimos
    let mut sum_min: u32 = subjects.iter().map(|s| s.min_minutes).sum();
    if sum_min > total_minutes {
        let factor = total_minutes as f32 / sum_min as f32;
        for s in &mut subjects {
            let new_min = ((s.min_minutes as f32) * factor).round() as u32;
            s.min_minutes = new_min.max(10);
        }
        sum_min = subjects.iter().map(|s| s.min_minutes).sum();
        if sum_min > total_minutes {
            subjects.sort_by_key(|s| s.priority);
            while sum_min > total_minutes && !subjects.is_empty() {
                let removed = subjects.remove(0);
                sum_min -= removed.min_minutes;
            }
        }
    }

    if subjects.is_empty() {
        return Vec::new();
    }

    let remaining = total_minutes.saturating_sub(sum_min);
    let total_weight: u32 = subjects.iter().map(|s| s.priority).sum();

    let mut blocks: Vec<Block> = subjects
        .iter()
        .map(|s| Block {
            subject_name: s.name.clone(),
            minutes: s.min_minutes,
        })
        .collect();

    // Distribui o resto proporcional ao peso
    if remaining > 0 && total_weight > 0 {
        let mut distributed = 0;
        for (idx, s) in subjects.iter().enumerate() {
            let extra =
                ((remaining as f32) * (s.priority as f32) / (total_weight as f32)).round() as u32;
            blocks[idx].minutes += extra;
            distributed += extra;
        }

        // 🔧 Aqui estava o problema: usamos len() dentro do índice
        if distributed < remaining {
            let mut i = 0;
            let len = blocks.len(); // pega o tamanho UMA VEZ só
            while distributed < remaining {
                let idx = i % len;
                blocks[idx].minutes += 1;
                distributed += 1;
                i += 1;
            }
        }
    }

    blocks.retain(|b| b.minutes >= 10);

    blocks
}

/// Converte blocos em horários reais para o frontend
fn blocks_to_ui_blocks(blocks: Vec<Block>, start_time: NaiveTime) -> Vec<UiBlock> {
    let mut current_time = start_time;
    let mut ui_blocks = Vec::new();

    for b in blocks {
        let end_time = current_time + chrono::Duration::minutes(b.minutes as i64);
        ui_blocks.push(UiBlock {
            start: current_time.format("%H:%M").to_string(),
            end: end_time.format("%H:%M").to_string(),
            subject: b.subject_name,
            minutes: b.minutes,
        });
        current_time = end_time;
    }

    ui_blocks
}
