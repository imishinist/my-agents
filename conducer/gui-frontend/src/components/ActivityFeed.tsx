import { useState } from 'react';
import type { Message } from '../types';

export function ActivityFeed({ messages }: { messages: Message[] }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div>
      <button
        className="text-sm font-medium text-gray-600 hover:text-gray-900"
        onClick={() => setExpanded(!expanded)}
      >
        {expanded ? '▼' : '▶'} Activity Feed ({messages.length})
      </button>
      {expanded && (
        <div className="mt-2 max-h-60 overflow-y-auto rounded-lg border border-gray-200 bg-white">
          {messages.length === 0 ? (
            <p className="p-3 text-xs text-gray-400">No activity yet</p>
          ) : (
            messages.map((m) => (
              <div key={m.id} className="border-b border-gray-100 px-3 py-2 last:border-0">
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-gray-400">
                    {new Date(m.timestamp).toLocaleTimeString()}
                  </span>
                  <span className="font-medium text-gray-700">
                    {m.source} → {m.destination}
                  </span>
                  <span className="rounded bg-gray-100 px-1.5 py-0.5 text-gray-500">
                    {m.type}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
