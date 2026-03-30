import type { Worker } from '../types';

const statusStyle: Record<string, string> = {
  idle: 'bg-gray-300',
  busy: 'bg-green-400',
  stalled: 'bg-red-400',
  offline: 'bg-gray-500',
};

export function WorkerPanel({ workers }: { workers: Worker[] }) {
  if (workers.length === 0) {
    return <p className="text-sm text-gray-400">No workers</p>;
  }

  return (
    <div className="flex flex-col gap-2">
      <h3 className="text-xs font-bold uppercase tracking-wide text-gray-500">Workers</h3>
      {workers.map((w) => (
        <div key={w.id} className="rounded-lg border border-gray-200 bg-white p-2">
          <div className="flex items-center gap-2">
            <span className={`h-2 w-2 rounded-full ${statusStyle[w.status] ?? ''}`} />
            <span className="text-xs font-medium text-gray-700">{w.id}</span>
          </div>
          <p className="mt-0.5 text-xs text-gray-400">{w.status}</p>
          {w.current_feature_id && (
            <p className="text-xs text-gray-500">{w.current_feature_id}</p>
          )}
        </div>
      ))}
    </div>
  );
}
