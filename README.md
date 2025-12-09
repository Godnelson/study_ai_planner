# 📘 Study AI Planner
### Planejador de estudos com IA — Rust • Axum • HTML/CSS • Docker

O **Study AI Planner** é um planejador diário de estudos que gera rotinas personalizadas usando:

- ⚡ Heurística local em **Rust**
- 🤖 Integração opcional com IA (ChatGPT)
- 🎨 Interface moderna em HTML + CSS puro
- 🐳 Deploy via Docker em Render, Fly.io e outros

É uma aplicação rápida, leve, inteligente e com design clean, ideal para uso pessoal e como projeto de portfólio.

---

## 🚀 Demonstração

🔗 **Link da aplicação**: *(adicione após o deploy)*  
📸 **Screenshot**: *(adicione aqui uma imagem depois)*

---

## ✨ Funcionalidades

### 🎯 Planejamento inteligente
- Distribuição de tempo por prioridade
- Mínimos por matéria
- Geração automática de blocos com início/fim
- Suporte à matéria foco

### 🤖 Modo IA (opcional)
Se você fornecer `OPENAI_API_KEY`:

- O backend envia dados ao ChatGPT
- Recebe uma rotina melhorada
- Inclui explicações e estrutura refinada

Sem IA → usa heurística local instantânea.

### 🎨 UI moderna
- Design elegante em dark mode
- Layout responsivo
- Componentes suaves e com boa hierarquia visual

### 🛠 Backend Rust + Axum
- Rápido e seguro
- Rota `/api/plan` (POST)
- Serve `/static/index.html` diretamente

### 🐳 Docker-ready
- Multi-stage build
- Imagem final leve
- Ideal para deploys automáticos

---

## 📦 Tecnologias

- Rust 1.85+
- Axum
- Tokio
- Serde
- Reqwest
- Docker
- HTML + CSS moderno

---

## 📁 Estrutura do Projeto

```
study_ai_planner/
│
├── src/
│   └── main.rs               # servidor Axum + rotas + heurística + chamada IA
│
├── static/
│   └── index.html            # interface web moderna
│
├── Dockerfile                # deploy via container
├── .gitignore
├── Cargo.toml
└── README.md
```

---

## ▶️ Rodando localmente

### 1. Clonar o repositório

```bash
git clone https://github.com/SEU-USUARIO/study_ai_planner
cd study_ai_planner
```

### 2. Rodar com Cargo

```bash
cargo run
```

Acesse:

```
http://localhost:3000
```

Opcional: criar `.env` para usar IA:

```
OPENAI_API_KEY=sk-xxxx
```

---

## 🐳 Rodando com Docker

### Build

```bash
docker build -t study-planner .
```

### Executar

```bash
docker run -p 3000:3000 study-planner
```

Abrir:

```
http://localhost:3000
```

---

## 🚀 Deploy no Render (recomendado)

1. Suba o repo no GitHub
2. No Render: **New Web Service**
3. Tipo: **Docker**
4. Deploy automático usando o Dockerfile
5. O Render define a variável `PORT` automaticamente

---

## 🔐 Variáveis de Ambiente

| Variável          | Descrição                                     | Obrigatória |
|------------------|-----------------------------------------------|-------------|
| `OPENAI_API_KEY` | Chave da API da OpenAI (modo IA)              | ❌ Opcional |
| `PORT`           | Porta definida pelo Render / Docker           | ❌ Opcional |

Sem `OPENAI_API_KEY`, o sistema usa heurística local.

---

## 📊 Heurística Interna (sem IA)

- Normaliza prioridades
- Calcula pesos proporcionais
- Ajusta blocos ao total de horas
- Garante mínimos por matéria
- Gera horários contínuos (ex: 08:00 → 09:20 → 10:10...)
- Ultra rápido: executa em microssegundos

---

## 🤝 Contribuindo

Pull Requests, Issues e sugestões são bem-vindos!

---

## 📜 Licença

MIT — fique à vontade para usar, modificar e estudar.

---

## ❤️ Autor

Desenvolvido por **Godnelson**  
Focado em Rust, performance, IA aplicada e interfaces clean.