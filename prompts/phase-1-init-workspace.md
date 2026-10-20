# Prompt — Fase 1: Inicialização do repositório FreeRusch

> Status: CONCLUÍDO. Branch `feat/init-freerusch-workspace` publicado no GitHub e PR #1 aberto
> em https://github.com/AltamiroAJ/FreeRusch/pull/1 (commit 773ed7c).

## Contexto e objetivo

Você é um agente de codificação encarregado de inicializar do zero o repositório **FreeRusch**, um software **puro em Rust** para cálculo estrutural de lajes de pontes pelo método de Rüsch (tabelas de Rüsch, trens-tipo TB-450/TB-240 conforme NBR 7188:2013, interpolação de tabelas, diagramas de momentos e emissão de memória de cálculo em HTML).

O projeto deve seguir a arquitetura e as convenções da **família Craft** (PhotoCraft, SolidCraft, CadCraft, GridCraft, etc.) e **reutilizar ao máximo as bibliotecas do projeto `craft-libs`** (`craft-core`, `craft-ui`, `craft-settings`, `craft-atomic`, `craft-print`) como base de UI, configurações, escrita atômica de arquivos e impressão/exportação.

Importante: o software T-Rüsch (https://www.tqs.com.br/apps/t-rusch/a6q52fh86z) é apenas **referência funcional** do domínio de engenharia; ele **não** deve ser usado como dependência nem copiado. O FreeRusch é uma reimplementação limpa e aberta.

## O que é a família Craft (resumo que você deve assumir)

- Softwares open source que são reimplementações limpas de aplicativos populares, construídos inteiramente em Rust ("pure Rust").
- Organizados como **Cargo Workspace** (monorepo), com bibliotecas compartilhadas em `crates/` e aplicativos finais em `apps/`.
- As bibliotecas compartilhadas vivem no repositório `craft-libs` (`https://github.com/storytold/craft-libs`) e incluem, entre outras:
  - `craft-core`: núcleo do framework GUI.
  - `craft-ui`: componentes de UI baseados em `egui` (botões, abas, ícones etc.).
  - `craft-settings`: gerenciamento de configurações persistidas em `config.toml`.
  - `craft-atomic`: escrita atômica de arquivos (segura contra corrupção).
  - `craft-print`: impressão/exportação de documentos.
- Os aplicativos da família (ex.: PhotoCraft) **consomem** essas bibliotecas como dependências Git fixadas em um commit.

## Tarefas que você deve executar

1. **Criar o workspace Cargo** na raiz do repositório, com `resolver = "3"`, `edition = "2024"`, licença `MIT OR Apache-2.0` e `repository` apontando para o repositório do FreeRusch.
2. **Criar os crates específicos do FreeRusch** em `crates/`, seguindo o padrão de nomenclatura da família (prefixo do projeto):
   - `crates/freerusch-core`: motor de cálculo puro (tabelas de Rüsch embutidas, interpolação, trens-tipo TB-450/TB-240, cálculo de esforços e geração de diagramas de momentos). **Sem dependências de UI.**
   - `crates/freerusch-report`: geração da memória de cálculo em HTML, usando `craft-print` e `craft-atomic` para escrita segura do arquivo.
   - (Opcional, se necessário) `crates/freerusch-ui`: componentes de UI específicos do domínio, construídos **sobre** `craft-ui`/`egui` quando o `craft-ui` não oferecer o componente pronto.
3. **Criar o aplicativo binário** em `apps/freerusch`, que:
   - Inicializa a GUI usando `craft-ui` (que é baseado em `egui`);
   - Carrega/salva configurações via `craft-settings` (`config.toml`);
   - Usa `craft-atomic` para gravação atômica de arquivos de projeto e `craft-print` para exportar a memória de cálculo.
4. **Configurar as dependências do `craft-libs`** no `[workspace.dependencies]` do `Cargo.toml` raiz, fixando um commit (rev) do branch padrão:
   ```toml
   [workspace.dependencies]
   craft-core     = { git = "https://github.com/storytold/craft-libs", rev = "<LATEST_COMMIT>" }
   craft-ui       = { git = "https://github.com/storytold/craft-libs", rev = "<LATEST_COMMIT>" }
   craft-settings = { git = "https://github.com/storytold/craft-libs", rev = "<LATEST_COMMIT>" }
   craft-atomic   = { git = "https://github.com/storytold/craft-libs", rev = "<LATEST_COMMIT>" }
   craft-print    = { git = "https://github.com/storytold/craft-libs", rev = "<LATEST_COMMIT>" }
   ```
   Resolva `<LATEST_COMMIT>` executando `git ls-remote https://github.com/storytold/craft-libs HEAD` e use o hash retornado (fixar o commit é obrigatório, conforme recomendação do próprio `craft-libs`).
5. **Adicionar `rust-toolchain.toml`** fixando o canal `stable` (compatível com edition 2024).
6. **Criar `.github/workflows/ci.yml`** rodando em `ubuntu-latest`, `windows-latest` e `macos-latest`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace` e `cargo build --workspace`.
7. **Criar um `README.md`** curto explicando o propósito do FreeRusch, a arquitetura (workspace + crates + app) e como executar (`cargo run -p freerusch`).
8. **Deixar o esqueleto compilando e testado**, com pelo menos:
   - Uma função pública em `freerusch-core` para calcular o momento fletor de uma laje de ponte a partir de uma tabela de Rüsch, com um teste unitário simples;
   - Um `main.rs` em `apps/freerusch` que abre uma janela mínima via `craft-ui` e exibe "FreeRusch" no título.

## Estrutura de arquivos esperada

```
freerusch/
├── Cargo.toml                     # workspace
├── rust-toolchain.toml
├── README.md
├── .gitignore
├── .github/workflows/ci.yml
├── crates/
│   ├── freerusch-core/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── freerusch-report/
│       ├── Cargo.toml
│       └── src/lib.rs
└── apps/
    └── freerusch/
        ├── Cargo.toml
        └── src/main.rs
```

## Regras e convenções

- **Pure Rust**: não introduzir dependências de C/C++ ou binários externos.
- **Crates específicos do FreeRusch** usam o prefixo `freerusch-`; as bibliotecas da família são consumidas como `craft-*`.
- **Não invente APIs do `craft-libs`**: inspecione o código-fonte e os exemplos do repositório `storytold/craft-libs` e use a API real do `craft-ui`, `craft-settings`, `craft-atomic` e `craft-print`. Se algum componente necessário não existir no `craft-ui`, implemente-o em `crates/freerusch-ui` **sobre** `egui`/`craft-ui`, sem duplicar o que já existe.
- **Erros** com `thiserror`; **logs** com `tracing` (opcional no esqueleto).
- **Documentação**: `///` em todos os itens públicos; `README` com exemplo de uso.
- **Formatação e lint**: rodar `cargo fmt`, `cargo clippy -- -D warnings` e `cargo test --workspace` antes de finalizar.
- **Commits e PR**: criar uma branch `feat/init-freerusch-workspace`, commitar em passos lógicos e abrir um Pull Request descrevendo a estrutura criada e como executar (`cargo run -p freerusch`).

## Critérios de aceite

1. `cargo build --workspace` compila sem warnings.
2. `cargo test --workspace` passa, incluindo ao menos um teste unitário em `freerusch-core` para o cálculo de momento fletor a partir de uma tabela de Rüsch.
3. `cargo clippy --workspace -- -D warnings` passa.
4. `cargo run -p freerusch` abre uma janela GUI mínima com o título "FreeRusch" via `craft-ui`.
5. As dependências do `craft-libs` estão fixadas em um commit (rev) válido e o projeto compila em Linux, Windows e macOS (CI verde).
6. O `README.md` explica propósito, arquitetura e execução.

## Observações finais

- Se o build do `craft-libs` falhar em alguma plataforma, registre a falha no PR e isole atrás de feature flag (`#[cfg]` ou `features` no `Cargo.toml`) em vez de remover a dependência.
- Priorize um esqueleto **mínimo, compilando e testado** sobre uma implementação completa; o domínio de engenharia (tabelas de Rüsch completas, trens-tipo etc.) pode vir em PRs seguintes, desde que a estrutura e as interfaces públicas já estejam definidas.
