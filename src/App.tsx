import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  Plus, 
  Trash2, 
  Activity, 
  Wifi, 
  WifiOff, 
  Settings,
  AlertCircle,
  Clock
} from 'lucide-react';
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

function App() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [statusMap, setStatusMap] = useState<Record<number, { is_online: boolean, last_check: string, latency?: number }>>({});
  const [isAdding, setIsAdding] = useState(false);
  const [newName, setNewName] = useState('');
  const [newIp, setNewIp] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDevices();

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

    return () => {
      unlistenCheck.then(u => u());
      unlistenTransition.then(u => u());
    };
  }, []);

  const fetchDevices = async () => {
    try {
      const result = await invoke<Device[]>('get_devices');
      setDevices(result);
    } catch (e) {
      console.error(e);
    }
  };

  const handleAdd = async () => {
    try {
      setError(null);
      await invoke('add_device', { name: newName, ip: newIp });
      setNewName('');
      setNewIp('');
      setIsAdding(false);
      fetchDevices();
    } catch (e: any) {
      setError(e.toString());
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

  return (
    <div className="container" style={{ padding: '1rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h1 style={{ fontSize: '1.25rem', fontWeight: 'bold', display: 'flex', alignItems: 'center', gap: '0.5rem', margin: 0 }}>
          <Activity size={24} color="#3b82f6" /> SCM Monitor
        </h1>
        <button 
          className="add-btn" 
          onClick={() => setIsAdding(!isAdding)}
          disabled={devices.length >= 4}
          style={{ 
            display: 'flex', 
            alignItems: 'center', 
            gap: '0.25rem', 
            backgroundColor: '#3b82f6', 
            color: 'white', 
            border: 'none', 
            padding: '0.5rem 1rem', 
            borderRadius: '0.375rem',
            cursor: devices.length >= 4 ? 'not-allowed' : 'pointer',
            opacity: devices.length >= 4 ? 0.5 : 1
          }}
        >
          <Plus size={18} /> Adicionar
        </button>
      </header>

      {devices.length >= 4 && !isAdding && (
        <div style={{ padding: '0.5rem', backgroundColor: '#fef3c7', borderLeft: '4px solid #f59e0b', fontSize: '0.875rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <AlertCircle size={14} /> Limite do MVP atingido (máx 4 dispositivos)
        </div>
      )}

      {isAdding && (
        <div style={{ backgroundColor: 'white', padding: '1rem', border: '1px solid #e2e8f0', borderRadius: '0.5rem', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
          <h3 style={{ margin: 0, fontSize: '1rem' }}>Novo Dispositivo</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
            <label style={{ fontSize: '0.75rem', fontWeight: 'bold', color: '#64748b' }}>NOME</label>
            <input 
              value={newName} 
              onChange={(e) => setNewName(e.target.value)} 
              placeholder="Ex: Servidor Principal"
              style={{ padding: '0.5rem', border: '1px solid #cbd5e1', borderRadius: '0.25rem' }}
            />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
            <label style={{ fontSize: '0.75rem', fontWeight: 'bold', color: '#64748b' }}>ENDEREÇO IP (IPv4)</label>
            <input 
              value={newIp} 
              onChange={(e) => setNewIp(e.target.value)} 
              placeholder="Ex: 1.1.1.1"
              style={{ padding: '0.5rem', border: '1px solid #cbd5e1', borderRadius: '0.25rem' }}
            />
          </div>
          {error && <div style={{ color: '#ef4444', fontSize: '0.75rem' }}>{error}</div>}
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.5rem', marginTop: '0.5rem' }}>
            <button 
              onClick={() => setIsAdding(false)}
              style={{ padding: '0.4rem 0.75rem', border: '1px solid #cbd5e1', backgroundColor: 'transparent', borderRadius: '0.25rem', cursor: 'pointer' }}
            >
              Cancelar
            </button>
            <button 
              onClick={handleAdd}
              style={{ padding: '0.4rem 0.75rem', backgroundColor: '#3b82f6', color: 'white', border: 'none', borderRadius: '0.25rem', cursor: 'pointer' }}
            >
              Salvar
            </button>
          </div>
        </div>
      )}

      <div className="device-list" style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {devices.length === 0 && !isAdding && (
          <div style={{ textAlign: 'center', padding: '3rem 0', color: '#94a3b8' }}>
            <Wifi size={48} strokeWidth={1} style={{ opacity: 0.5, marginBottom: '0.5rem' }} />
            <p style={{ margin: 0 }}>Nenhum dispositivo monitorado.</p>
          </div>
        )}

        {devices.map(device => {
          const status = statusMap[device.id];
          const isOnline = status?.is_online;
          
          return (
            <div key={device.id} style={{ 
              backgroundColor: 'white', 
              padding: '1rem', 
              border: '1px solid #e2e8f0', 
              borderRadius: '0.5rem', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'space-between'
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                <div style={{ 
                  width: '40px', 
                  height: '40px', 
                  borderRadius: '100%', 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'center',
                  backgroundColor: isOnline === undefined ? '#f1f5f9' : (isOnline ? '#dcfce7' : '#fee2e2'),
                  color: isOnline === undefined ? '#94a3b8' : (isOnline ? '#166534' : '#991b1b')
                }}>
                  {isOnline === false ? <WifiOff size={20} /> : <Wifi size={20} />}
                </div>
                <div>
                  <div style={{ fontWeight: 'bold', fontSize: '1rem' }}>{device.name}</div>
                  <div style={{ fontSize: '0.8rem', color: '#64748b' }}>{device.ip}</div>
                </div>
              </div>
              
              <div style={{ display: 'flex', alignItems: 'center', gap: '1.5rem' }}>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '0.1rem' }}>
                  {status ? (
                    <>
                      <div style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
                        <Clock size={12} /> {status.last_check}
                      </div>
                      <div style={{ fontSize: '0.875rem', fontWeight: 'bold', color: isOnline ? '#16a34a' : '#dc2626' }}>
                        {isOnline ? `${status.latency?.toFixed(1)} ms` : 'OFFLINE'}
                      </div>
                    </>
                  ) : (
                    <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>Iniciando...</div>
                  )}
                </div>

                <button 
                  onClick={() => handleDelete(device.id)}
                  style={{ 
                    backgroundColor: 'transparent', 
                    border: 'none', 
                    color: '#94a3b8', 
                    cursor: 'pointer',
                    padding: '0.5rem'
                  }}
                  onMouseOver={(e) => (e.currentTarget.style.color = '#ef4444')}
                  onMouseOut={(e) => (e.currentTarget.style.color = '#94a3b8')}
                >
                  <Trash2 size={18} />
                </button>
              </div>
            </div>
          );
        })}
      </div>

      <footer style={{ marginTop: 'auto', borderTop: '1px solid #e2e8f0', paddingTop: '1rem', display: 'flex', justifyContent: 'space-between', alignItems: 'center', color: '#94a3b8', fontSize: '0.7rem' }}>
        <div>SCM-TOOL SYSTEM MONITOR</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
          <Settings size={12} /> v1.0.0 (Windows)
        </div>
      </footer>
    </div>
  );
}

export default App;
