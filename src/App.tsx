import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Badge,
  Body1Strong,
  Button,
  Caption1,
  Card,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  DialogTrigger,
  Divider,
  Field,
  Input,
  MessageBar,
  MessageBarBody,
  makeStyles,
  tokens,
} from '@fluentui/react-components';
import {
  Add24Regular,
  Clock24Regular,
  Delete24Regular,
  Edit24Regular,
  Settings16Regular,
} from '@fluentui/react-icons';
import './App.css';

interface Device {
  id: number;
  name: string;
  ip: string;
  is_active: boolean;
}

interface CheckEvent {
  device_id: number;
  is_online: boolean;
  latency_ms: number;
}

interface AppError {
  id: number;
  source: string;
  message: string;
  timestamp?: string;
}

interface NotificationErrorEvent {
  source: string;
  message: string;
}

const useStyles = makeStyles({
  page: {
    width: '100%',
    minHeight: '100vh',
    padding: '16px',
    display: 'flex',
    justifyContent: 'center',
    backgroundColor: tokens.colorNeutralBackground2,
  },
  shell: {
    width: '100%',
    maxWidth: '760px',
    minHeight: 'calc(100vh - 32px)',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '12px',
  },
  titleWrap: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
  },
  title: {
    margin: 0,
    fontSize: '22px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
  },
  subtitle: {
    color: tokens.colorNeutralForeground3,
  },
  dialogContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
  },
  emptyCard: {
    textAlign: 'center',
    padding: '24px',
    color: tokens.colorNeutralForeground3,
  },
  deviceCard: {
    padding: '12px',
  },
  deviceRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '12px',
    width: '100%',
  },
  deviceMain: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    minWidth: 0,
    flex: 1,
  },
  deviceInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    minWidth: 0,
  },
  ipText: {
    color: tokens.colorNeutralForeground3,
  },
  right: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  statusBox: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: '4px',
  },
  checkTime: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    color: tokens.colorNeutralForeground3,
  },
  statusOnline: {
    color: tokens.colorPaletteGreenForeground1,
    fontWeight: 600,
  },
  statusOffline: {
    color: tokens.colorPaletteRedForeground1,
    fontWeight: 600,
  },
  initializing: {
    color: tokens.colorNeutralForeground3,
  },
  footer: {
    marginTop: 'auto',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    color: tokens.colorNeutralForeground3,
    paddingTop: '4px',
  },
  footerRight: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
  },
});

function App() {
  const styles = useStyles();
  const [devices, setDevices] = useState<Device[]>([]);
  const [statusMap, setStatusMap] = useState<Record<number, { is_online: boolean, last_check: string, latency?: number }>>({});
  const [isAdding, setIsAdding] = useState(false);
  const [newName, setNewName] = useState('');
  const [newIp, setNewIp] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [editingDeviceId, setEditingDeviceId] = useState<number | null>(null);
  const [editName, setEditName] = useState('');
  const [editIp, setEditIp] = useState('');
  const [editError, setEditError] = useState<string | null>(null);
  const [notificationWarning, setNotificationWarning] = useState<string | null>(null);

  const fetchDevices = useCallback(async () => {
    try {
      const result = await invoke<Device[]>('get_devices');
      setDevices(result);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    const frameId = window.requestAnimationFrame(() => {
      void fetchDevices();
    });

    void invoke<AppError[]>('get_app_errors')
      .then((errors) => {
        const latestNotificationError = errors.find((entry) => entry.source === 'NOTIFICATION');
        if (latestNotificationError) {
          setNotificationWarning(latestNotificationError.message);
        }
      })
      .catch((e) => {
        console.error(e);
      });

    const unlistenCheck = listen<CheckEvent>('check-event', (event) => {
      setStatusMap(prev => ({
        ...prev,
        [event.payload.device_id]: {
          is_online: event.payload.is_online,
          last_check: new Date().toLocaleTimeString(),
          latency: event.payload.latency_ms
        }
      }));
    });

    const unlistenTransition = listen('transition-event', () => {
      // Re-fetch or just let check-event update status
      fetchDevices();
    });

    const unlistenNotificationError = listen<NotificationErrorEvent>('notification-error-event', (event) => {
      setNotificationWarning(event.payload.message);
    });

    return () => {
      window.cancelAnimationFrame(frameId);
      unlistenCheck.then(u => u());
      unlistenTransition.then(u => u());
      unlistenNotificationError.then(u => u());
    };
  }, [fetchDevices]);

  const resetAddForm = () => {
    setNewName('');
    setNewIp('');
    setError(null);
  };

  const handleAdd = async () => {
    try {
      setError(null);
      await invoke('add_device', { name: newName, ip: newIp });
      resetAddForm();
      setIsAdding(false);
      fetchDevices();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await invoke('remove_device', { id });
      fetchDevices();
    } catch (e) {
      console.error(e);
    }
  };

  const closeEditForm = () => {
    setEditingDeviceId(null);
    setEditName('');
    setEditIp('');
    setEditError(null);
  };

  const openEditForm = (device: Device) => {
    setEditingDeviceId(device.id);
    setEditName(device.name);
    setEditIp(device.ip);
    setEditError(null);
  };

  const handleEdit = async () => {
    if (editingDeviceId === null) {
      return;
    }

    try {
      setEditError(null);
      await invoke('update_device', {
        id: editingDeviceId,
        name: editName,
        ip: editIp,
      });
      closeEditForm();
      fetchDevices();
    } catch (e: unknown) {
      setEditError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleDialogOpenChange = (_event: unknown, data: { open: boolean }) => {
    setIsAdding(data.open);
    if (!data.open) {
      resetAddForm();
    }
  };

  const handleEditDialogOpenChange = (_event: unknown, data: { open: boolean }) => {
    if (!data.open) {
      closeEditForm();
    }
  };

  const maxReached = devices.length >= 4;
  const canSave = newName.trim().length > 0 && newIp.trim().length > 0;
  const canSaveEdit = editName.trim().length > 0 && editIp.trim().length > 0;

  return (
    <div className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.header}>
          <div className={styles.titleWrap}>
            <h1 className={styles.title}>SCM Monitor</h1>
            <Caption1 className={styles.subtitle}>Monitor de conectividade em tempo real</Caption1>
          </div>

          <Dialog open={isAdding} onOpenChange={handleDialogOpenChange}>
            <DialogTrigger disableButtonEnhancement>
              <Button appearance="primary" icon={<Add24Regular />} disabled={maxReached}>
                Adicionar dispositivo
              </Button>
            </DialogTrigger>
            <DialogSurface>
              <DialogBody>
                <DialogTitle>Novo dispositivo</DialogTitle>
                <DialogContent className={styles.dialogContent}>
                  <Field label="Nome">
                    <Input
                      value={newName}
                      onChange={(_, data) => setNewName(data.value)}
                      placeholder="Ex: Servidor Principal"
                    />
                  </Field>
                  <Field label="Endereço IP (IPv4)">
                    <Input
                      value={newIp}
                      onChange={(_, data) => setNewIp(data.value)}
                      placeholder="Ex: 1.1.1.1"
                    />
                  </Field>
                  {error && (
                    <MessageBar intent="error">
                      <MessageBarBody>{error}</MessageBarBody>
                    </MessageBar>
                  )}
                </DialogContent>
                <DialogActions>
                  <DialogTrigger disableButtonEnhancement>
                    <Button appearance="secondary">Cancelar</Button>
                  </DialogTrigger>
                  <Button appearance="primary" onClick={handleAdd} disabled={!canSave}>
                    Salvar
                  </Button>
                </DialogActions>
              </DialogBody>
            </DialogSurface>
          </Dialog>
        </header>

        {maxReached && !isAdding && (
          <MessageBar intent="warning">
            <MessageBarBody>Limite do MVP atingido (máx 4 dispositivos).</MessageBarBody>
          </MessageBar>
        )}

        {notificationWarning && (
          <MessageBar intent="warning">
            <MessageBarBody>{notificationWarning}</MessageBarBody>
          </MessageBar>
        )}

        <Dialog open={editingDeviceId !== null} onOpenChange={handleEditDialogOpenChange}>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>Editar dispositivo</DialogTitle>
              <DialogContent className={styles.dialogContent}>
                <Field label="Nome">
                  <Input
                    value={editName}
                    onChange={(_, data) => setEditName(data.value)}
                    placeholder="Ex: Servidor Principal"
                  />
                </Field>
                <Field label="Endereço IP (IPv4)">
                  <Input
                    value={editIp}
                    onChange={(_, data) => setEditIp(data.value)}
                    placeholder="Ex: 1.1.1.1"
                  />
                </Field>
                {editError && (
                  <MessageBar intent="error">
                    <MessageBarBody>{editError}</MessageBarBody>
                  </MessageBar>
                )}
              </DialogContent>
              <DialogActions>
                <DialogTrigger disableButtonEnhancement>
                  <Button appearance="secondary">Cancelar</Button>
                </DialogTrigger>
                <Button appearance="primary" onClick={handleEdit} disabled={!canSaveEdit}>
                  Salvar alterações
                </Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>

        <div className={styles.list}>
          {devices.length === 0 && !isAdding && (
            <Card>
              <div className={styles.emptyCard}>
                <Body1Strong>Nenhum dispositivo monitorado.</Body1Strong>
                <Caption1>Clique em "Adicionar dispositivo" para começar.</Caption1>
              </div>
            </Card>
          )}

          {devices.map(device => {
            const status = statusMap[device.id];
            const isOnline = status?.is_online;
            const statusText = status ? (isOnline ? 'Online' : 'Offline') : 'Iniciando';
            const statusColor: 'informative' | 'success' | 'danger' = status
              ? (isOnline ? 'success' : 'danger')
              : 'informative';

            return (
              <Card key={device.id} className={styles.deviceCard}>
                <div className={styles.deviceRow}>
                  <div className={styles.deviceMain}>
                    <Badge color={statusColor} appearance="filled">
                      {statusText}
                    </Badge>
                    <div className={styles.deviceInfo}>
                      <Body1Strong>{device.name}</Body1Strong>
                      <Caption1 className={styles.ipText}>{device.ip}</Caption1>
                    </div>
                  </div>

                  <div className={styles.right}>
                    {status ? (
                      <div className={styles.statusBox}>
                        <Caption1 className={styles.checkTime}>
                          <Clock24Regular fontSize={12} />
                          {status.last_check}
                        </Caption1>
                        <Caption1 className={isOnline ? styles.statusOnline : styles.statusOffline}>
                          {isOnline ? `${status.latency?.toFixed(1) ?? '-'} ms` : 'OFFLINE'}
                        </Caption1>
                      </div>
                    ) : (
                      <Caption1 className={styles.initializing}>Aguardando checagem...</Caption1>
                    )}

                    <Button
                      aria-label={`Editar ${device.name}`}
                      appearance="subtle"
                      icon={<Edit24Regular />}
                      onClick={() => openEditForm(device)}
                    />
                    <Button
                      aria-label={`Remover ${device.name}`}
                      appearance="subtle"
                      icon={<Delete24Regular />}
                      onClick={() => handleDelete(device.id)}
                    />
                  </div>
                </div>
              </Card>
            );
          })}
        </div>

        <Divider />
        <footer className={styles.footer}>
          <Caption1>SCM-TOOL SYSTEM MONITOR</Caption1>
          <div className={styles.footerRight}>
            <Settings16Regular />
            <Caption1>v1.0.0 (Windows)</Caption1>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default App;
