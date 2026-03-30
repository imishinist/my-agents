import type { Epic, Feature, Worker, ActionItem, Message } from '../types';

const BASE = import.meta.env.VITE_API_BASE || '';

async function fetchJSON<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${url}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export const api = {
  listEpics: () => fetchJSON<Epic[]>('/api/epics'),
  createEpic: (title: string, description: string) =>
    fetchJSON<Epic>('/api/epics', {
      method: 'POST',
      body: JSON.stringify({ title, description }),
    }),
  getEpic: (id: string) => fetchJSON<Epic>(`/api/epics/${id}`),
  retryEpic: (id: string) =>
    fetch(`${BASE}/api/epics/${id}/retry`, { method: 'POST' }),

  listFeatures: () => fetchJSON<Feature[]>('/api/features'),
  getFeature: (id: string) => fetchJSON<Feature>(`/api/features/${id}`),

  listWorkers: () => fetchJSON<Worker[]>('/api/workers'),

  listActions: () => fetchJSON<ActionItem[]>('/api/actions'),
  respondToAction: (id: string, body: { answer?: string; granted?: boolean; notes?: string }) =>
    fetch(`${BASE}/api/actions/${id}/respond`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),

  listMessages: () => fetchJSON<Message[]>('/api/messages'),
};
