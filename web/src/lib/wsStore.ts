import { writable } from 'svelte/store';

export type WsStatus = 'connected' | 'connecting' | 'disconnected';

interface WsState {
  status: WsStatus;
  authenticated: boolean;
}

function createWsStore() {
  const { subscribe, set, update } = writable<WsState>({
    status: 'disconnected',
    authenticated: false,
  });

  return {
    subscribe,
    setConnecting: () => update(s => ({ ...s, status: 'connecting' })),
    setConnected: (authenticated = false) => set({ status: 'connected', authenticated }),
    setDisconnected: () => set({ status: 'disconnected', authenticated: false }),
    setAuthenticated: (authenticated: boolean) => update(s => ({ ...s, authenticated })),
  };
}

export const wsStore = createWsStore();
