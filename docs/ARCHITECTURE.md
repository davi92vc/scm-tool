# Arquitetura do Sistema - SCM Tool

O **SCM-TOOL SYSTEM MONITOR** é construído usando **Tauri v2**, **Rust** no backend e **React + TypeScript** no frontend.

## Diagramas e Componentes

### 1. Backend (Rust)

- **Tauri Application:** Gerencia o ciclo de vida da janela do Windows e a integração com o tray.
- **Tray Status Aggregator:** Calcula estado agregado dos dispositivos e atualiza ícone da tray em runtime (verde quando todos online, vermelho quando há offline, neutro no estado inicial/sem devices).
- **MonitoringEngine:** Um executor assíncrono (usa `tokio`) que spawnava tarefas independentes para cada dispositivo monitorado.
  - **Algoritmo de Checagem:**
    - **Status Online:** Verifica conforme intervalo configurado (padrão 10 segundos).
    - **Status Offline:** Se falhar 1 vez, muda para Offline.
    - **Recuperação:** Verifica conforme intervalo configurado para offline (padrão 2 segundos).
- **ICMP Service (`surge-ping`):** Usado para enviar pings reais de rede.
- **SQLite Database (`sqlx`):** Armazena configurações de dispositivos, histórico de `checks`, `transitions`, `app_errors` e `app_settings`.
  - **WAL Mode:** Configurado para suportar concorrência de leitura/escrita rápida.
  - **Purge Automático:** Uma tarefa limpa registros com mais de 30 dias na inicialização.
- **Autostart Plugin (`tauri-plugin-autostart`):** Controla habilitar/desabilitar inicialização com o sistema e mantém espelho de estado em `app_settings`.

### 2. Frontend (React)

- **State Management:** Usa `useState` e `useEffect` do React para gerenciar os dispositivos.
- **Tauri Events:** Escuta eventos assíncronos enviados pelo backend (`check-event` e `transition-event`) via `listen`.
- **Tauri Commands:** Chama funções do backend (`get_devices`, `add_device`, `remove_device`, `get_settings`, `update_settings`) via `invoke`.
- **UI Components:** Estilizados com CSS moderno e ícones do `lucide-react`.

### 3. Fluxo de Dados

1. **Frontend:** Adiciona um dispositivo via `add_device` command.
2. **Backend:** Valida o IP, salva no banco e notifica o `MonitoringEngine`.
3. **MonitoringEngine:** Inicia o loop de pings para o novo dispositivo.
4. **Backend:** A cada ping bem-sucedido ou falha, emite um `check-event` para o frontend.
5. **Backend:** Se houver mudança de estado (Up/Down), emite um `transition-event` e dispara uma **Windows Toast Notification**.
6. **Backend:** Também dispara notificação no primeiro status detectado de cada dispositivo.
7. **Backend + Frontend:** Se o envio da notificação nativa falhar, o backend registra em `app_errors` e emite `notification-error-event` para exibir aviso visual na UI.

## Segurança e Performance

- **Threads Seguras:** Toda a checagem de rede é assíncrona e não bloqueia a UI.
- **SQLite Concorrência:** O banco usa o modo WAL para permitir leitura simultânea enquanto as tarefas de monitoramento escrevem os logs.
- **Limites de MVP:** Limitado a 4 dispositivos para garantir performance e simplicidade no primeiro lançamento.
