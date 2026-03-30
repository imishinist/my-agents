import { useEffect, useRef } from 'react';

const BASE = import.meta.env.VITE_API_BASE || '';

export function useSSE(onEvent: (type: string, data: string) => void) {
  const cbRef = useRef(onEvent);
  cbRef.current = onEvent;

  useEffect(() => {
    const es = new EventSource(`${BASE}/api/events`);

    es.onmessage = (e) => {
      cbRef.current('message', e.data);
    };

    // Listen for typed events
    const types = [
      'epic.created', 'epic.decomposed', 'epic.completed',
      'review.completed', 'worker.stalled',
      'features.unblocked', 'action.responded',
      'clarification.answered', 'clarification.escalated',
    ];

    for (const t of types) {
      es.addEventListener(t, (e) => {
        cbRef.current(t, (e as MessageEvent).data);
      });
    }

    return () => es.close();
  }, []);
}
