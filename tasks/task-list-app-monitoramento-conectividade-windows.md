# Task List: App de Monitoramento de Conectividade (Windows)

**Source PRD:** task/prd-app-monitoramento-conectividade-windows.md  
**Architecture ref:** ADR-001, ADR-002, ADR-003, ADR-004  
**Target:** Mid-level Developer  
**Estimated Duration:** 52 horas totais

---

## Phase 1 · Architecture Design

### Inputs Confirmados

- [x] Business objectives and success metrics (PRD seções 2 e 8)
- [x] Known constraints (Tauri v2, Windows 10/11, SQLite/similar, Sprint 1, baixo orçamento)
- [x] Current system overview (greenfield)
- [x] Quality attribute priorities (Reliability > Performance > Maintainability > Security > Cost)

### 1) Context Diagram (C4 L1)

```text
[Usuário administrativo não técnico]
        │
        ▼
[App Monitoramento de Conectividade (Tauri v2, Windows)]
        ├──► [Dispositivos de rede (ICMP ping)]
        ├──► [Windows OS APIs (notificação, tray, autostart)]
        └──► [Banco local SQLite (ou equivalente)]
```

### 2) Container Diagram (C4 L2)

| Name              | Technology                  | Responsibility                                                               | Interfaces                                 |
| ----------------- | --------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------ |
| Frontend Desktop  | TypeScript + UI Web (Tauri) | CRUD de dispositivos, exibição de status, ações de bandeja                   | Tauri Commands/Events                      |
| Core App          | Tauri v2 + Rust             | Orquestra ciclo de monitoramento, regras de domínio e integração OS          | IPC interno, APIs Windows                  |
| Monitoring Engine | Rust async runtime          | Ping periódico, cálculo de transição online/offline, deduplicação de eventos | Interface interna com Core App             |
| Persistence Store | SQLite (preferencial)       | Persistir dispositivos, checagens, transições e erros                        | SQL adapter/repositório                    |
| OS Integrations   | Plugins/APIs Tauri v2       | Notificação nativa, system tray, auto-start                                  | Windows notification center, tray, startup |

### 3) Architecture Decision Records (ADRs)

### ADR-001: Modular Monolith em Tauri v2

**Status:** Accepted  
**Context:** MVP simples, 1 dev, Sprint 1, baixo orçamento.  
**Decision:** Usar aplicativo único (modular monolith) com fronteiras internas claras entre UI, domínio, integrações e persistência.  
**Alternatives Considered:** Microserviços locais; serviço Windows separado. Rejeitados por complexidade e custo operacional.  
**Consequences:** Entrega mais rápida e manutenção simples no MVP; extração futura possível se limites provarem estáveis.  
**Risks:** Acoplamento interno crescer sem disciplina de módulos.

### ADR-002: Persistência Local com SQLite Preferencial

**Status:** Accepted  
**Context:** Requisito de histórico local, confiabilidade e baixo overhead.  
**Decision:** SQLite como padrão; fallback para armazenamento local equivalente somente se bloqueio técnico no ambiente.  
**Alternatives Considered:** JSON files; armazenamento em memória. Rejeitados por risco de integridade e perda de histórico.  
**Consequences:** Consultas e retenção viáveis; requer migração/schema versionado.  
**Risks:** Lock de escrita e corrupção por encerramento abrupto.

### ADR-003: Engine de Monitoramento no Backend Rust

**Status:** Accepted  
**Context:** Necessidade de execução contínua em background e detecção em até 3s.  
**Decision:** Executar scheduler e checagens ICMP no backend Rust, UI apenas como cliente de estado/evento.  
**Alternatives Considered:** Loop no frontend; execução por scripts externos. Rejeitados por confiabilidade e manutenção.  
**Consequences:** Maior estabilidade e controle de concorrência; mais código Rust no MVP.  
**Risks:** Erros de concorrência e sincronização de estado.

### ADR-004: Modelo de Segurança MVP (sem login/autorização de app)

**Status:** Accepted  
**Context:** PRD define sem gestão de usuários/login no MVP local single-user.  
**Decision:** Não implementar autenticação/autorização de aplicação no MVP; adotar boundary de segurança pelo usuário do Windows e validação estrita de entrada.  
**Alternatives Considered:** Login local com PIN/senha no app. Rejeitado por escopo e valor baixo no MVP.  
**Consequences:** Menor atrito e complexidade; deve ser documentado como limitação explícita.  
**Risks:** Uso por usuário não autorizado no mesmo perfil de sessão Windows.

### 4) Non-Functional Requirements

| Attribute       | Target                                                    | Measurement                                    |
| --------------- | --------------------------------------------------------- | ---------------------------------------------- |
| Reliability     | Monitoramento contínuo por 8h sem interrupção             | Heartbeat sem gaps > 15s                       |
| Performance     | Detecção de queda ≤ 3s em 95% dos casos                   | Diferença entre timestamp de falha e transição |
| Security        | Validação rigorosa de input + boundary do usuário do SO   | Testes de validação + checklist de segurança   |
| Cost            | Rodar local sem infra externa paga                        | Zero dependência cloud no MVP                  |
| Maintainability | Módulos internos com contratos claros e testes essenciais | Cobertura de testes nas regras críticas        |
| Data retention  | 30 dias de histórico operacional                          | Job de purge automático validado               |

### 5) Critical Path & Risks

1. **Risco de bloqueio ICMP em ambiente alvo (M):** pode gerar falso offline.  
   **Mitigação:** classificar erro técnico em log e diferenciar timeout/reject; abrir OQ para fallback TCP futuro.
2. **Risco de lock/erro no banco local (L):** perda de eventos de monitoramento.  
   **Mitigação:** modo WAL, retries de escrita e fila curta em memória para flush.
3. **Risco de duplicação de notificação em oscilação (M):** ruído para o usuário.  
   **Mitigação:** regra de deduplicação por transição confirmada.
4. **Risco de falha em auto-start/tray (M):** monitoramento não inicia conforme esperado.  
   **Mitigação:** estado visível da configuração + log de erro + fallback manual.
5. **Risco de crescimento de acoplamento no monolito (M):** manutenção degradada.  
   **Mitigação:** módulos com interfaces explícitas e ADRs atualizadas.

---

## Phase 2 · Task Generation

## Setup & Infrastructure

- [x] **T001 · Inicializar projeto Tauri v2 para Windows**  
      **Goal:** Ter base executable do app desktop com build local funcionando.  
      **Sub-tasks:**
  - [x] Criar projeto Tauri v2 com stack frontend definida.
  - [x] Validar toolchain Windows (Rust, WebView2, build tools).
  - [x] Ajustar configuração base de app (nome, janela, ícone inicial).
        **Acceptance Criteria:** App inicia em modo dev e build local sem erro.
        **Verification:** Executar fluxo de build/dev e abrir app no Windows.
        **Dependencies:** None

- [ ] **T002 · Configurar pipeline mínimo de qualidade local/CI**  
      **Goal:** Garantir validação automática de lint/test/build em cada alteração crítica.  
      **Sub-tasks:**
  - [ ] Definir scripts de lint/test/build para frontend e backend.
  - [ ] Criar workflow de CI para rodar em push/PR.
  - [ ] Falhar pipeline em erro de lint/test/build.
        **Acceptance Criteria:** Pipeline executa e reprova em erro proposital.
        **Verification:** Rodar CI com commit de teste controlado.
        **Dependencies:** T001

## Data Layer

- [ ] **T003 · Definir schema SQLite e migração inicial**  
      **Goal:** Estruturar persistência para dispositivos e eventos operacionais.  
      **Sub-tasks:**
  - [ ] Criar tabelas: devices, checks, transitions, errors.
  - [ ] Aplicar constraints (IP único ativo, campos obrigatórios).
  - [ ] Versionar migração inicial.
        **Acceptance Criteria:** Banco inicializado com schema versionado e constraints ativas.
        **Verification:** Executar migração e validar estrutura/constraints por query.
        **Dependencies:** T001

- [ ] **T004 · Implementar retenção de 30 dias com purge automático**  
      **Goal:** Controlar crescimento do histórico sem intervenção manual.  
      **Sub-tasks:**
  - [ ] Implementar job de purge na inicialização e ciclo diário.
  - [ ] Preservar dados de dispositivos ativos.
  - [ ] Registrar métricas de registros removidos.
        **Acceptance Criteria:** Registros antigos (>30 dias) são removidos automaticamente.
        **Verification:** Inserir dados antigos de teste e validar purge.
        **Dependencies:** T003

- [ ] **T005 · Criar camada de repositório com validação de persistência**  
      **Goal:** Isolar SQL e impedir gravação de dados inválidos.  
      **Sub-tasks:**
  - [ ] Implementar operações CRUD de devices.
  - [ ] Implementar inserção de checks/transitions/errors.
  - [ ] Tratar erros de lock/retry de forma padronizada.
        **Acceptance Criteria:** Operações persistem corretamente com tratamento de erro consistente.
        **Verification:** Testes de repositório para sucesso e falha.
        **Dependencies:** T003

## Domain Logic

- [ ] **T006 · Regras de cadastro e limite do MVP**  
      **Goal:** Garantir regras de negócio de cadastro antes de persistir.  
      **Sub-tasks:**
  - [ ] Validar IPv4 estritamente.
  - [ ] Bloquear IP duplicado ativo.
  - [ ] Bloquear inclusão acima de 4 dispositivos.
        **Acceptance Criteria:** Entradas inválidas sempre rejeitadas com erro claro.
        **Verification:** Testes unitários cobrindo casos válidos e inválidos.
        **Dependencies:** T005

- [ ] **T007 · Scheduler dinâmico de monitoramento (10s/2s)**  
      **Goal:** Executar checagens contínuas por estado do dispositivo.  
      **Sub-tasks:**
  - [ ] Implementar ciclo de checagem por dispositivo.
  - [ ] Aplicar intervalo de 10s para online e 2s para offline.
  - [ ] Persistir resultado e latência de cada checagem.
        **Acceptance Criteria:** Intervalos aplicados corretamente em runtime.
        **Verification:** Logs e testes de tempo de ciclo.
        **Dependencies:** T006

- [ ] **T008 · Motor de transição e deduplicação de eventos**  
      **Goal:** Emitir evento apenas em mudança real de estado.  
      **Sub-tasks:**
  - [ ] Detectar transições online→offline e offline→online.
  - [ ] Deduplicar notificações em estado estável.
  - [ ] Tratar oscilação com janela de confirmação mínima.
        **Acceptance Criteria:** Razão notificação/transição igual a 1.0 em cenário controlado.
        **Verification:** Teste de oscilação com sequência de status simulada.
        **Dependencies:** T007

## API / Service Layer

- [ ] **T009 · Expor comandos Tauri para CRUD e monitoramento**  
      **Goal:** Fornecer contratos estáveis entre UI e backend.  
      **Sub-tasks:**
  - [ ] Criar comandos para listar/criar/editar/remover devices.
  - [ ] Criar comando para iniciar/parar monitoramento.
  - [ ] Padronizar payload de erro para UI.
        **Acceptance Criteria:** Todos os fluxos principais funcionam via comandos públicos.
        **Verification:** Testes de integração comando→domínio→repositório.
        **Dependencies:** T006, T007

- [ ] **T010 · Expor stream de eventos de status para UI**  
      **Goal:** Atualizar interface sem polling manual do frontend.  
      **Sub-tasks:**
  - [ ] Publicar eventos de status, transição e erro.
  - [ ] Definir contrato de evento tipado e versionado.
  - [ ] Garantir entrega ordenada por dispositivo.
        **Acceptance Criteria:** UI recebe atualizações em tempo real com ordem consistente.
        **Verification:** Teste de integração com listener e sequência validada.
        **Dependencies:** T008

## Integrations

- [ ] **T011 · Integrar ICMP adapter para Windows**  
      **Goal:** Realizar ping com latência e motivo técnico de falha.  
      **Sub-tasks:**
  - [ ] Implementar adapter de ICMP compatível com Windows.
  - [ ] Mapear erros técnicos (timeout, unreachable, blocked).
  - [ ] Encapsular adapter atrás de interface de domínio.
        **Acceptance Criteria:** Resultado de ping retorna sucesso/falha + latência + motivo.
        **Verification:** Testes com hosts válidos e inválidos.
        **Dependencies:** T007

- [ ] **T012 · Integrar notificações nativas Windows**  
      **Goal:** Notificar usuário em transições com conteúdo padronizado.  
      **Sub-tasks:**
  - [ ] Implementar adapter de notificação nativa.
  - [ ] Enviar toast apenas em mudança de estado.
  - [ ] Registrar falha de envio sem interromper monitoramento.
        **Acceptance Criteria:** Toast aparece em transição e não duplica em estado estável.
        **Verification:** Cenário manual + logs de envio.
        **Dependencies:** T008

- [ ] **T013 · Integrar tray e auto-start no Windows**  
      **Goal:** Permitir operação em background com controle pelo menu da bandeja.  
      **Sub-tasks:**
  - [ ] Configurar ícone e menu (Abrir, Iniciar com Windows, Sair).
  - [ ] Interceptar fechamento para minimizar na bandeja.
  - [ ] Implementar toggle de auto-start com feedback de erro.
        **Acceptance Criteria:** Fluxos de minimizar, restaurar, sair e auto-start funcionam.
        **Verification:** Testes manuais no Windows com reinício do sistema.
        **Dependencies:** T001, T009

## User Interface

- [ ] **T014 · Construir tela principal e estados obrigatórios**  
      **Goal:** Entregar UI única e simples para usuário não técnico.  
      **Sub-tasks:**
  - [ ] Implementar layout principal com lista de dispositivos.
  - [ ] Implementar estados: vazio, monitorando, erro validação, falha notificação.
  - [ ] Destacar ação primária “Adicionar dispositivo” no estado vazio.
        **Acceptance Criteria:** Todos os estados do PRD são navegáveis e legíveis.
        **Verification:** Checklist de UI por estado.
        **Dependencies:** T009, T010

- [ ] **T015 · Implementar fluxo de cadastro/edição/remoção com validação**  
      **Goal:** Permitir CRUD completo com feedback claro de erro.  
      **Sub-tasks:**
  - [ ] Criar formulário com validação de IPv4 e mensagens objetivas.
  - [ ] Mostrar erro para IP duplicado e limite de 4 dispositivos.
  - [ ] Atualizar lista em tempo real após operação.
        **Acceptance Criteria:** Usuário conclui CRUD sem quebrar regras de domínio.
        **Verification:** Testes de componente e fluxo manual guiado.
        **Dependencies:** T006, T014

- [ ] **T016 · Exibir status em tempo real e último sucesso/falha**  
      **Goal:** Dar visibilidade operacional sem ações extras do usuário.  
      **Sub-tasks:**
  - [ ] Consumir eventos de status do backend.
  - [ ] Atualizar badges e timestamps por dispositivo.
  - [ ] Exibir erros operacionais de forma não intrusiva.
        **Acceptance Criteria:** Status e timestamps refletem eventos em tempo real.
        **Verification:** Simular queda/retorno e validar atualização visual.
        **Dependencies:** T010, T015

## Observability

- [ ] **T017 · Logging estruturado de domínio e integrações**  
      **Goal:** Permitir diagnóstico rápido de falhas em produção local.  
      **Sub-tasks:**
  - [ ] Definir formato único de logs (nível, origem, device, erro).
  - [ ] Logar transições, falhas ICMP, falhas notificação e auto-start.
  - [ ] Correlacionar eventos de monitoramento por device_id.
        **Acceptance Criteria:** Eventos críticos aparecem com contexto suficiente para troubleshooting.
        **Verification:** Validar logs em cenários de falha simulada.
        **Dependencies:** T007, T011, T012, T013

- [ ] **T018 · Métricas internas para KPIs do PRD**  
      **Goal:** Medir sucesso (detecção, estabilidade, alertas perdidos).  
      **Sub-tasks:**
  - [ ] Coletar métrica de tempo de detecção por transição.
  - [ ] Coletar heartbeat e gaps de monitoramento.
  - [ ] Coletar comparação entre transições e notificações enviadas.
        **Acceptance Criteria:** Métricas permitem calcular KPI-1 a KPI-3 automaticamente.
        **Verification:** Rodar consulta de métricas no banco e validar resultados.
        **Dependencies:** T004, T008, T012, T017

## Testing

- [ ] **T019 · Testes unitários das regras de domínio**  
      **Goal:** Garantir robustez das regras críticas (validação e transição).  
      **Sub-tasks:**
  - [ ] Cobrir validação IPv4, limite de 4 e IP duplicado.
  - [ ] Cobrir scheduler 10s/2s.
  - [ ] Cobrir deduplicação de notificação.
        **Acceptance Criteria:** Testes unitários passando para todas as regras críticas.
        **Verification:** Executar suíte unitária com relatório de sucesso.
        **Dependencies:** T006, T007, T008

- [ ] **T020 · Testes de integração backend + SQLite + comandos**  
      **Goal:** Validar fluxos ponta a ponta no backend local.  
      **Sub-tasks:**
  - [ ] Testar CRUD completo via comandos Tauri.
  - [ ] Testar persistência de checks/transitions/errors.
  - [ ] Testar comportamento em lock/retry e falhas de integração.
        **Acceptance Criteria:** Fluxos de integração estáveis sem regressão de dados.
        **Verification:** Executar suíte de integração automatizada.
        **Dependencies:** T009, T005, T011

- [ ] **T021 · Smoke E2E Windows para fluxos críticos de usuário**  
      **Goal:** Validar cenário real de uso do MVP por usuário não técnico.  
      **Sub-tasks:**
  - [ ] Script de teste para cadastro e início de monitoramento.
  - [ ] Teste de minimizar para bandeja, restaurar e sair.
  - [ ] Teste de notificação em transição online/offline.
        **Acceptance Criteria:** Fluxos críticos executados sem falhas em ambiente Windows.
        **Verification:** Evidência de execução (logs/checklist) em sessão de teste.
        **Dependencies:** T013, T016, T020

## Documentation

- [ ] **T022 · Publicar ADRs e guia de arquitetura do MVP**  
      **Goal:** Registrar decisões para manter evolução controlada.  
      **Sub-tasks:**
  - [ ] Documentar ADR-001 a ADR-004 em pasta de arquitetura.
  - [ ] Adicionar diagrama de contexto e containers em markdown.
  - [ ] Registrar trade-offs e critérios de extração futura.
        **Acceptance Criteria:** Documentação arquitetural revisável e alinhada ao código.
        **Verification:** Revisão técnica cruzada com checklist ADR.
        **Dependencies:** T001, T003, T007

- [ ] **T023 · Criar runbook operacional e guia de troubleshooting**  
      **Goal:** Permitir operação e suporte básicos sem conhecimento profundo.  
      **Sub-tasks:**
  - [ ] Documentar instalação, execução e configuração de auto-start.
  - [ ] Documentar falhas comuns (ICMP bloqueado, notificação indisponível, DB lock).
  - [ ] Documentar rollback local (desabilitar feature/config e restaurar DB backup).
        **Acceptance Criteria:** Runbook cobre operação diária e incidentes mais prováveis.
        **Verification:** Execução de simulação guiada por pessoa não autora.
        **Dependencies:** T017, T021

- [ ] **T024 · Checklist de segurança MVP (auth/authz/input validation)**  
      **Goal:** Cobrir gate de segurança sem violar o escopo do PRD.  
      **Sub-tasks:**
  - [ ] Documentar decisão “sem autenticação/autorização de app” e boundary pelo usuário do SO.
  - [ ] Validar e testar entradas em todos os pontos de escrita.
  - [ ] Registrar ameaças conhecidas e controles compensatórios do MVP.
        **Acceptance Criteria:** Checklist aprovado com evidências de validação de input e decisão de auth/authz registrada.
        **Verification:** Revisão de segurança baseada no documento e testes de input.
        **Dependencies:** T006, T022

---

## Task Dependency Graph

```text
T001 → T002
T001 → T003 → T004 → T018
T003 → T005 → T006 → T007 → T008
T006 + T007 → T009 → T010 → T016
T007 → T011
T008 → T012
T001 + T009 → T013
T009 + T010 → T014 → T015 → T016
T007 + T011 + T012 + T013 → T017 → T023
T006 + T007 + T008 → T019
T009 + T005 + T011 → T020 → T021
T001 + T003 + T007 → T022 → T024
```

---

## Coverage Mapping (PRD → Tasks)

- FR-01..FR-03 → T001, T022
- FR-04..FR-07 → T003, T004, T005
- FR-08..FR-11 → T006, T015
- FR-12..FR-16 → T007, T008, T016, T018
- FR-17..FR-19 → T012, T016, T018
- FR-20..FR-25 → T013, T021
- FR-26 → T022 (evolução planejada)

---

## Deferred / Out of Scope

- Fallback TCP para hosts com ICMP bloqueado: adiado para V2 (depende de validação operacional).
- Suporte Linux/macOS: fora do MVP conforme PRD.
- Gestão de usuários/login/perfis no app: fora do MVP, manter ADR-004.
- Dashboard avançado e exportação: fora do MVP para preservar simplicidade.

---

## Open Questions

- OQ-1: Histórico de eventos terá visualização dedicada no MVP? — Owner: Product Owner — Due: 04 de março de 2026.
- OQ-2: Retenção final será 30 ou 60 dias? — Owner: Operações — Due: 04 de março de 2026.
- OQ-3: Auto-start oficial via plugin Tauri ou registro do usuário? — Owner: Tech Lead — Due: 06 de março de 2026.

---

## Quality Gates Check

- [x] Todo requisito do PRD mapeado para ao menos uma task.
- [x] Toda decisão ADR refletida em tasks.
- [x] Sem loop de dependências.
- [x] Riscos críticos possuem tasks de mitigação.
- [x] Tasks de observabilidade para engine e banco local.
- [x] Task de segurança cobre auth/authz (decisão documentada) e validação de entrada.
- [x] Tasks dimensionadas em blocos de 1–4 horas.
