import type { Feature } from '../types';

const priorityColor: Record<string, string> = {
  critical: 'bg-red-100 text-red-800',
  high: 'bg-orange-100 text-orange-800',
  medium: 'bg-blue-100 text-blue-800',
  low: 'bg-gray-100 text-gray-600',
};

export function FeatureCard({ feature }: { feature: Feature }) {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-3 shadow-sm">
      <div className="mb-1 flex items-center justify-between">
        <span className={`rounded px-1.5 py-0.5 text-xs font-medium ${priorityColor[feature.priority] ?? ''}`}>
          {feature.priority}
        </span>
        {feature.worker_id && (
          <span className="text-xs text-gray-400">{feature.worker_id}</span>
        )}
      </div>
      <h4 className="text-sm font-semibold text-gray-900">{feature.title}</h4>
      {feature.blocked_reason && (
        <p className="mt-1 text-xs text-red-500">⛔ {feature.blocked_reason}</p>
      )}
      {feature.pr_number && (
        <p className="mt-1 text-xs text-gray-400">PR #{feature.pr_number}</p>
      )}
    </div>
  );
}
