# Rataria 🐀

Um TUI para gerenciamento de alvos em Pentest, Red Team e Bug Bounty.  
Organiza todas as informações coletadas durante o recon de um ou mais alvos — subdomains, IPs, ASNs, URLs, tecnologias e screenshots — em um banco local criptografado.

---

## Funcionalidades

- **Multi-engagement** — gerencie múltiplos alvos e programas simultaneamente
- **Banco criptografado** — SQLite + SQLCipher com AES-256, protegido por senha master
- **Timeout de sessão** — banco fecha automaticamente após 5 minutos de inatividade
- **Importação flexível** — aceita JSON da ratazana, JSON genérico ou TXT simples
- **Exportação** — gera relatório Markdown completo do engagement
- **Preview de screenshots** — visualização inline via protocolo Kitty
- **TDD** — 219+ testes cobrindo toda a camada de dados

---

## Instalação

### Dependências

```bash
# Arch Linux
sudo pacman -S base-devel clang

# Ubuntu/Debian
sudo apt install build-essential libclang-dev clang
```

### Compilar

```bash
git clone https://github.com/seu-usuario/rataria
cd rataria
cargo build --release
./target/release/rataria
```

---

## Como usar

### Navegação geral

| Tecla | Ação |
|-------|------|
| `↑↓` ou `jk` | Navegar |
| `Enter` | Selecionar / Confirmar |
| `Esc` | Voltar / Cancelar |
| `?` | Ajuda (atalhos do contexto atual) |
| `q` | Sair (na tela inicial) |

### Subdomains

| Tecla | Ação |
|-------|------|
| `N` | Novo subdomain |
| `S` | Ciclar status |
| `O` | Editar notas |
| `F` | Filtrar por status |
| `D` | Deletar (pede confirmação) |

### Dashboard

| Tecla | Ação |
|-------|------|
| `E` | Exportar relatório Markdown |

### Screenshots

| Tecla | Ação |
|-------|------|
| `Enter` | Preview inline (requer Kitty) |
| `O` | Abrir com visualizador do sistema |

---

## Importação

### TXT simples

Um subdomain por linha. Linhas começando com `#` são ignoradas.

```
# gerado pelo subfinder
api.empresa.com
admin.empresa.com
dev.empresa.com
```

Na tela de import:
- **Caminho:** `/path/to/subs.txt`
- **Target:** `empresa.com` (obrigatório para TXT)
- **Engagement:** nome do engagement (opcional)

---

### JSON genérico

Formato mínimo:

```json
{
  "target": "empresa.com",
  "subdomains": ["api.empresa.com", "admin.empresa.com"]
}
```

Formato completo:

```json
{
  "target": "empresa.com",
  "engagement": "Bug Bounty Q1",
  "subdomains": [
    {
      "subdomain": "api.empresa.com",
      "status_code": 200,
      "title": "API Principal",
      "technologies": [
        { "name": "Nginx", "version": "1.24" },
        { "name": "React", "version": null }
      ],
      "urls": [
        { "url": "https://api.empresa.com/v1/users", "url_type": "endpoint" },
        { "url": "https://api.empresa.com/app.js", "url_type": "javascript" },
        { "url": "https://api.empresa.com/search?q=test", "url_type": "parameter" }
      ]
    },
    "dev.empresa.com"
  ],
  "ips": ["192.168.1.1"],
  "asns": [
    { "asn": "AS12345", "org": "Empresa XPTO Ltda" }
  ]
}
```

**Campos disponíveis:**

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `target` | string | ✅ | Domínio principal |
| `engagement` | string | ❌ | Nome do engagement (usa o atual se vazio) |
| `subdomains` | array | ✅ | Lista de subdomains (string ou objeto) |
| `ips` | array | ❌ | Lista de IPs |
| `asns` | array | ❌ | Lista de ASNs |

**Tipos de URL:**

| Valor | Descrição |
|-------|-----------|
| `endpoint` | Endpoints de API |
| `javascript` | Arquivos JS |
| `parameter` | URLs com parâmetros (`?param=value`) |
| `other` | Qualquer outro tipo |

---

### JSON da Ratazana

Se você usar a [ratazana](https://github.com/seu-usuario/ratazana) para recon automatizado, o output já está no formato correto para importação direta. O Rataria detecta automaticamente pelo campo `rataria_version`.

Exemplo:

```json
{
  "rataria_version": "1.0",
  "target": "empresa.com",
  "engagement_name": "Bug Bounty Q1",
  "timestamp": "2025-04-24T10:00:00Z",
  "subdomains": [
    {
      "subdomain": "api.empresa.com",
      "status_code": 200,
      "title": "API Principal",
      "technologies": [
        { "name": "Nginx", "version": "1.24" }
      ],
      "urls": [
        { "url": "https://api.empresa.com/v1/login", "url_type": "endpoint" }
      ],
      "sources": ["subfinder", "httpx"]
    }
  ],
  "ips": ["192.168.1.1"],
  "asns": [
    { "asn": "AS12345", "org": "Empresa XPTO Ltda" }
  ]
}
```

---

## Dados armazenados

```
~/.local/share/rataria/rataria.db  ← banco criptografado
~/rataria_<engagement>_<timestamp>.md  ← relatório exportado
```

---

## Segurança

- Banco criptografado com **SQLCipher + AES-256**
- Senha derivada via **Argon2id**
- Chave nunca escrita em disco
- **Timeout de sessão** de 5 minutos por padrão
- Confirmação antes de qualquer exclusão

---

## Licença

MIT
