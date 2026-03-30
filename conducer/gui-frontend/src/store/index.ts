import { create } from 'zustand';
import type { Epic, Feature, Worker, ActionItem, Message } from '../types';
import { api } from '../hooks/useAPI';

interface Store {
  epics: Epic[];
  features: Feature[];
  workers: Worker[];
  actions: ActionItem[];
  messages: Message[];
  loading: boolean;

  refresh: () => Promise<void>;
  createEpic: (title: string, description: string) => Promise<void>;
  retryEpic: (id: string) => Promise<void>;
  respondToAction: (id: string, body: { answer?: string; granted?: boolean; notes?: string }) => Promise<void>;
}

export const useStore = create<Store>((set, get) => ({
  epics: [],
  features: [],
  workers: [],
  actions: [],
  messages: [],
  loading: false,

  refresh: async () => {
    set({ loading: true });
    const [epics, features, workers, actions, messages] = await Promise.all([
      api.listEpics(),
      api.listFeatures(),
      api.listWorkers(),
      api.listActions(),
      api.listMessages(),
    ]);
    set({ epics, features, workers, actions, messages, loading: false });
  },

  createEpic: async (title, description) => {
    await api.createEpic(title, description);
    await get().refresh();
  },

  retryEpic: async (id) => {
    await api.retryEpic(id);
    await get().refresh();
  },

  respondToAction: async (id, body) => {
    await api.respondToAction(id, body);
    await get().refresh();
  },
}));
