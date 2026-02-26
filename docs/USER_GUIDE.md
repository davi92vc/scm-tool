# Guia do Usuário - SCM Tool

O **SCM-TOOL SYSTEM MONITOR** é uma aplicação para Windows desenvolvida para monitorar a conectividade de servidores e dispositivos de rede em tempo real.

## Como Usar

### 1. Adicionar Dispositivo

- Clique em **Adicionar**.
- Digite o **Nome** (ex: "Servidor Web").
- Digite o **Endereço IPv4** (ex: "192.168.1.1").
- Clique em **Salvar**.
- _Nota: Durante o MVP, há um limite de 4 dispositivos._

### 2. Monitoramento

- **Online (Verde):** O dispositivo está respondendo aos pings.
- **Offline (Vermelho):** O dispositivo falhou em 3 tentativas consecutivas.
- **Latência:** Exibe o tempo de resposta em milissegundos.
- **Timestamp:** Mostra o horário da última verificação.

### 3. Notificações

- Se um dispositivo mudar de Online para Offline ou vice-versa, o Windows exibirá uma notificação no canto da tela.

### 4. Segundo Plano (Tray)

- Ao fechar a janela (clicando no X), a aplicação continua rodando em segundo plano.
- Você pode encontrar o ícone na área de notificação do Windows (perto do relógio).
- Clique com o botão direito para abrir ou sair da aplicação.

### 5. Histórico e Limpeza

- A aplicação mantém logs e histórico de conectividade por **30 dias**.
- Dados mais antigos de 30 dias são removidos automaticamente para economizar espaço.

## Requisitos de Sistema

- Windows 10 ou 11.
- Conexão de Internet estável para o monitoramento externo.
- Permissões de administrador (para algumas funções de rede ICMP se necessário).
