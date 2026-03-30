import { useEffect } from 'react';
import { useStore } from './store';
import { useSSE } from './hooks/useSSE';
import { KanbanBoard } from './components/KanbanBoard';
import { WorkerPanel } from './components/WorkerPanel';
import { ActionQueue } from './components/ActionQueue';
import { EpicForm } from './components/EpicForm';
import { ActivityFeed } from './components/ActivityFeed';

export default function App() {
  const { epics, features, workers, actions, messages, loading, refresh, createEpic, retryEpic, respondToAction } = useStore();

  useEffect(() => { refresh(); }, [refresh]);

  useSSE(() => { refresh(); });

  const activeEpic = epics.find((e) => e.status === 'active') ?? epics[0];

  return (
    <div className="flex h-screen flex-col bg-gray-100 text-gray-900">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-gray-200 bg-white px-6 py-3">
        <h1 className="text-lg font-bold">conducer</h1>
        <div className="flex items-center gap-4">
          {activeEpic && (
            <span className="text-sm text-gray-500">
              {activeEpic.title} — <span className={`font-medium ${activeEpic.status === 'error' ? 'text-red-600' : ''}`}>{activeEpic.status}</span>
            </span>
          )}
          {activeEpic?.status === 'error' && activeEpic.last_error && (
            <div className="flex items-center gap-2">
              <span className="max-w-md truncate text-xs text-red-500" title={activeEpic.last_error}>
                {activeEpic.last_error}
              </span>
              <button
                className="rounded bg-red-600 px-2 py-1 text-xs text-white hover:bg-red-700"
                onClick={() => retryEpic(activeEpic.id)}
              >
                Retry
              </button>
            </div>
          )}
          {actions.length > 0 && (
            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-red-500 text-xs font-bold text-white">
              {actions.length}
            </span>
          )}
        </div>
      </header>

      {/* Body */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="w-48 shrink-0 overflow-y-auto border-r border-gray-200 bg-white p-4">
          <WorkerPanel workers={workers} />
        </aside>

        {/* Main */}
        <main className="flex flex-1 flex-col gap-4 overflow-y-auto p-6">
          {loading && <p className="text-sm text-gray-400">Loading...</p>}

          <KanbanBoard features={features} />

          <ActionQueue actions={actions} onRespond={respondToAction} />

          <ActivityFeed messages={messages} />
        </main>
      </div>

      {/* Footer */}
      <footer className="border-t border-gray-200 bg-white px-6 py-3">
        <EpicForm onSubmit={createEpic} />
      </footer>
    </div>
  );
}
