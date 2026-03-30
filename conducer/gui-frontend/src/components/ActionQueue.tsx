import { useState } from 'react';
import type { ActionItem } from '../types';

interface Props {
  actions: ActionItem[];
  onRespond: (id: string, body: { answer?: string; granted?: boolean; notes?: string }) => void;
}

export function ActionQueue({ actions, onRespond }: Props) {
  const [answers, setAnswers] = useState<Record<string, string>>({});

  if (actions.length === 0) return null;

  return (
    <div className="rounded-lg border border-amber-200 bg-amber-50 p-4">
      <h3 className="mb-3 text-sm font-bold text-amber-800">
        Action Queue ({actions.length})
      </h3>
      <div className="flex flex-col gap-3">
        {actions.map((a) => (
          <div key={a.id} className="rounded border border-amber-100 bg-white p-3">
            <div className="flex items-center gap-2">
              <span className="text-sm">{a.action_type === 'escalation' ? '⚠️' : '🔐'}</span>
              <span className="text-sm font-medium text-gray-800">{a.title}</span>
              <span className={`ml-auto rounded px-1.5 py-0.5 text-xs ${
                a.urgency === 'critical' ? 'bg-red-100 text-red-700' :
                a.urgency === 'high' ? 'bg-orange-100 text-orange-700' :
                'bg-gray-100 text-gray-600'
              }`}>
                {a.urgency}
              </span>
            </div>
            <p className="mt-1 text-xs text-gray-600">{a.question}</p>
            {a.action_type === 'escalation' ? (
              <div className="mt-2 flex gap-2">
                <input
                  className="flex-1 rounded border border-gray-300 px-2 py-1 text-xs"
                  placeholder="Answer..."
                  value={answers[a.id] ?? ''}
                  onChange={(e) => setAnswers({ ...answers, [a.id]: e.target.value })}
                />
                <button
                  className="rounded bg-blue-600 px-3 py-1 text-xs text-white hover:bg-blue-700"
                  onClick={() => onRespond(a.id, { answer: answers[a.id] })}
                >
                  Answer
                </button>
              </div>
            ) : (
              <div className="mt-2 flex gap-2">
                <button
                  className="rounded bg-green-600 px-3 py-1 text-xs text-white hover:bg-green-700"
                  onClick={() => onRespond(a.id, { granted: true })}
                >
                  Approve
                </button>
                <button
                  className="rounded bg-red-600 px-3 py-1 text-xs text-white hover:bg-red-700"
                  onClick={() => onRespond(a.id, { granted: false })}
                >
                  Deny
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
